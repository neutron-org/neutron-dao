use cosmos_sdk_proto::cosmos::{
    authz::v1beta1::MsgExec, bank::v1beta1::MsgSend, base::v1beta1::Coin as ProtoCoin,
};
use cosmos_sdk_proto::traits::Message;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, AnyMsg, BalanceResponse, BankQuery, Binary, Coin, CosmosMsg, Decimal,
    Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
use prost_types::Any;
use std::collections::HashSet;

use crate::error::ContractError;
use crate::error::ContractError::FlashloanAlreadyActive;
use crate::msg::{BorrowerInterface, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{ActiveLoan, Config, ACTIVE_LOAN, CONFIG};

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-flashloans";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Used to identify a reply to the /cosmos.bank.v1beta1.MsgSend message
/// that we execute immediately after receiving a loan request in the
/// RequestLoan handler.
pub const AUTHZ_BANK_SEND_REPLY_ID: u64 = 0;

/// Used to identify a reply to the ProcessLoan message that we send to
/// the borrower after transferring them the loan.
pub const BORROWER_HANDLER_REPLY_ID: u64 = 1;

pub const BANK_MSG_SEND_TYPE_URL: &str = "/cosmos.bank.v1beta1.MsgSend";
pub const AUTHZ_MSG_EXEC_TYPE_URL: &str = "/cosmos.authz.v1beta1.MsgExec";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Signifies that we start with no active loan
    ACTIVE_LOAN.save(deps.storage, &None)?;

    let owner_address = deps.api.addr_validate(msg.owner.as_str())?;
    let source_address = deps.api.addr_validate(msg.source.as_str())?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner_address,
            fee_rate: msg.fee_rate,
            source: source_address,
        },
    )?;

    Ok(Response::new()
        .add_attribute("owner", msg.owner.to_string())
        .add_attribute("fee_rate", msg.fee_rate.to_string())
        .add_attribute("source", msg.source.to_string())
        .add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            source,
            fee_rate,
        } => execute_update_config(deps, info, owner, source, fee_rate),
        ExecuteMsg::RequestLoan { amount } => execute_request_loan(deps, env, info, amount),
    }
}

/// Updates the config with values provided by the owner.
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    source: Option<String>,
    fee_rate: Option<Decimal>,
) -> Result<Response<NeutronMsg>, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Required to ensure that a malicious owner can't change the config while the
    // loan is active
    if ACTIVE_LOAN.load(deps.storage)?.is_some() {
        return Err(FlashloanAlreadyActive {});
    }

    if let Some(new_owner) = owner {
        let owner_address = deps.api.addr_validate(new_owner.as_str())?;
        config.owner = owner_address;
    }
    if let Some(new_source) = source {
        let source_address = deps.api.addr_validate(new_source.as_str())?;
        config.source = source_address;
    }
    // No fee rate validation is required here because we can properly process
    // any valid Decimal number
    if let Some(new_fee_rate) = fee_rate {
        config.fee_rate = new_fee_rate;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_config")
        .add_attribute("owner", config.owner.to_string())
        .add_attribute("source", config.source.to_string())
        .add_attribute("fee_rate", config.fee_rate.to_string()))
}

/// This handler ensures there is no active loan, validates the loan amount (no duplicate or zero
/// coins), calculates the expected balance (current balance + fee) of the source after repayment,
/// records the loan details in storage. If the `source` does not have the requested amount of
/// funds, an error will be returned. Finally, it instructs the source to send the requested amount
/// to the borrower via `authz`, encapsulated in a `stargate message`. This message is submitted as
/// a submessage with a `reply_on_success` strategy, meaning if it fails, the transaction is
/// reverted.
pub fn execute_request_loan(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Vec<Coin>,
) -> Result<Response<NeutronMsg>, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // Reentrancy guard
    if ACTIVE_LOAN.load(deps.storage)?.is_some() {
        return Err(FlashloanAlreadyActive {});
    }

    // Check that the amount does not have duplicate or zero coins
    validate_amount(amount.clone())?;

    // Getting the current balances of the source is necessary to make sure that the loan
    // was returned, and the fee was paid.
    let pre_loan_balances = get_pre_loan_balances(&deps, config.source.clone(), amount.clone())?;

    // Calculate the fee to be paid by the borrower for receiving the loan
    let fee = calculate_fee(amount.clone(), config.fee_rate)?;

    // Calculate the expected balance of the source after the loan has been returned
    // and the fee has been paid as (requested amount + fee).
    let expected_balances = calculate_expected_balances(pre_loan_balances, fee.clone())?;

    // Save all the information necessary to continue processing the loan request in the reply()
    // handler.
    ACTIVE_LOAN.save(
        deps.storage,
        &Some(ActiveLoan {
            borrower: info.sender.clone(),
            amount: amount.clone(),
            fee,
            expected_balances,
        }),
    )?;

    // Send a (stargate -> authz -> bank) /cosmos.bank.v1beta1.MsgSend submessage with
    // reply_on_success strategy (we want the transaction to be simply reverted in case of an
    // error).
    let msg_send = get_stargate_authz_bank_send_msg(env, config.source, info.sender, amount);
    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(msg_send, AUTHZ_BANK_SEND_REPLY_ID))
        .add_attribute("action", "execute_request_loan"))
}

// Check that the amount does not have duplicate or zero coins
fn validate_amount(amount: Vec<Coin>) -> Result<(), ContractError> {
    let mut denoms: HashSet<String> = HashSet::new();
    for coin in amount {
        if coin.amount.eq(&Uint128::zero()) {
            return Err(ContractError::ZeroRequested { denom: coin.denom });
        }

        if denoms.contains(&coin.denom) {
            return Err(ContractError::DuplicateDenoms {});
        }

        denoms.insert(coin.denom);
    }

    Ok(())
}

/// Returns the list of current balances on the contract's account for the coins that were
/// requested by the borrower. If any actual coin balance is lower than the requested amount,
/// returns an error.
fn get_pre_loan_balances(
    deps: &DepsMut,
    source: Addr,
    requested_amount: Vec<Coin>,
) -> Result<Vec<Coin>, ContractError> {
    // Filter all balances leaving only the balances of the requested coins
    let mut pre_loan_balances: Vec<Coin> = vec![];
    for requested_coin in requested_amount {
        // Prepare the query
        let requested_coin_balance_query = BankQuery::Balance {
            address: source.to_string(),
            denom: requested_coin.denom.clone(),
        };
        // Get the response for the requested coin
        let balance_response: BalanceResponse =
            deps.querier.query(&requested_coin_balance_query.into())?;

        // If the source has enough of the requested coin, remember the balance
        if requested_coin.amount.le(&balance_response.amount.amount) {
            pre_loan_balances.push(balance_response.amount.clone())
        } else {
            // If the source doesn't have (enough of) the requested coin, return an error
            return Err(ContractError::InsufficientFunds {
                denom: requested_coin.denom,
            });
        }
    }

    Ok(pre_loan_balances)
}

/// Calculates the fee by multiplying each of the requested assets by fee_rate.
fn calculate_fee(
    requested_amount: Vec<Coin>,
    fee_rate: Decimal,
) -> Result<Vec<Coin>, ContractError> {
    let mut fee: Vec<Coin> = Vec::with_capacity(requested_amount.len());
    for coin in requested_amount {
        let coin_fee = Coin::new(coin.amount.checked_mul_floor(fee_rate)?.u128(), coin.denom);
        fee.push(coin_fee)
    }

    Ok(fee)
}

// Sums the pre_loan_balances with the fee.
// WARNING: this function assumes that the input vectors are of the same length,
// and that the order of the denoms is the same.
fn calculate_expected_balances(
    pre_loan_balances: Vec<Coin>,
    fee: Vec<Coin>,
) -> Result<Vec<Coin>, ContractError> {
    let mut expected_balances: Vec<Coin> = Vec::with_capacity(pre_loan_balances.len());
    for (index, coin) in pre_loan_balances.iter().enumerate() {
        expected_balances.push(Coin::new(
            coin.amount.checked_add(fee[index].amount)?.u128(),
            coin.denom.clone(),
        ))
    }

    Ok(expected_balances)
}

/// A simple function to build the (stargate -> authz -> bank) /cosmos.bank.v1beta1.MsgSend message.
fn get_stargate_authz_bank_send_msg(
    env: Env,
    source: Addr,
    borrower: Addr,
    amount: Vec<Coin>,
) -> CosmosMsg<NeutronMsg> {
    // First we create the bank MsgSend
    let bank_send_msg = MsgSend {
        from_address: source.to_string(),
        to_address: borrower.to_string(),
        amount: amount
            .iter()
            .map(|x| ProtoCoin {
                denom: x.denom.clone(),
                amount: x.amount.to_string(),
            })
            .collect(),
    };

    // Then we wrap it in an authz message
    let authz_msg_exec = MsgExec {
        grantee: env.contract.address.to_string(),
        msgs: vec![Any {
            type_url: BANK_MSG_SEND_TYPE_URL.to_string(),
            value: bank_send_msg.encode_to_vec(),
        }],
    };

    // Then we wrap the authz message in a stargate message, because there is
    // no custom support for authz messages in CosmWasm.
    let stargate_authz_msg_exec: CosmosMsg<NeutronMsg> = CosmosMsg::Any(AnyMsg {
        type_url: AUTHZ_MSG_EXEC_TYPE_URL.to_string(),
        value: Binary::new(authz_msg_exec.encode_to_vec()),
    });

    stargate_authz_msg_exec
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        AUTHZ_BANK_SEND_REPLY_ID => {
            // If we are here, the money is already on the borrower's account. This means that
            // we can proceed to call the borrower's handler.

            let config: Config = CONFIG.load(deps.storage)?;
            let active_loan = must_get_active_loan(&deps)?;

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: active_loan.borrower.to_string(),
                msg: to_json_binary(&BorrowerInterface::ProcessLoan {
                    // We set the return address to the source address
                    return_address: config.source.to_string(),
                    loan_amount: active_loan.amount,
                    fee: active_loan.fee,
                })?,
                funds: vec![],
            });
            // We use the reply_on_success strategy (we want the transaction to be simply reverted
            // in case of an error).
            Ok(Response::new()
                .add_submessage(SubMsg::reply_on_success(msg, BORROWER_HANDLER_REPLY_ID))
                .add_attribute("action", "reply_authz_bank_send"))
        }
        BORROWER_HANDLER_REPLY_ID => {
            // If we are here, the borrower smart contract has successfully executed the
            // ProcessLoan message, and probably returned the funds to our contract together with
            // the additional fee.

            let config: Config = CONFIG.load(deps.storage)?;
            let active_loan = must_get_active_loan(&deps)?;

            // Check that the borrower returned the loan and paid the fee
            check_expected_balances(deps.as_ref(), config.source, active_loan.expected_balances)?;

            // Set the active loan to None, thus making ourselves ready for the next loan
            ACTIVE_LOAN.save(deps.storage, &None)?;

            Ok(Response::new().add_attribute("action", "reply_borrower_handler"))
        }
        _ => Err(ContractError::UnknownReplyID { id: msg.id }),
    }
}

fn check_expected_balances(
    deps: Deps,
    source: Addr,
    expected_balances: Vec<Coin>,
) -> Result<(), ContractError> {
    // For each of the expected coin balances, check that the current balance of the source
    // matches the expectations. We require **exactly** the expected amount (loan amount + fee)
    // to be transferred back to the source, not more, not less.
    for expected_coin in expected_balances {
        // Prepare the query
        let expected_coin_balance_query = BankQuery::Balance {
            address: source.to_string(),
            denom: expected_coin.denom.clone(),
        };
        // Get the response (all balances)
        let balance_response: BalanceResponse =
            deps.querier.query(&expected_coin_balance_query.into())?;

        // The amount needs to be exactly what we expect it to be.
        if !expected_coin.amount.eq(&balance_response.amount.amount) {
            return Err(ContractError::IncorrectPayback {});
        }
    }

    Ok(())
}

// Loads the information about the current loan, returns an error if the information
// is missing.
fn must_get_active_loan(deps: &DepsMut) -> Result<ActiveLoan, ContractError> {
    let active_loan = ACTIVE_LOAN.load(deps.storage)?;
    if let Some(active_loan) = active_loan {
        Ok(active_loan)
    } else {
        Err(ContractError::UnexpectedState {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
