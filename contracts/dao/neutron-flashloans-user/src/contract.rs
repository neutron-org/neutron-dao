#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;

use crate::error::ContractError;
use crate::msg::FlashloansExecuteMsg::RequestLoan;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{EXECUTION_MODE, FLASHLOANS_CONTRACT};

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-flashloans-user";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const MODE_RETURN_LOAN: u64 = 0;
pub const MODE_WITHHOLD_LOAN: u64 = 1;
pub const MODE_RETURN_LOAN_MORE_THAN_NECESSARY: u64 = 2;
pub const MODE_REQUEST_ANOTHER_LOAN_RECURSIVELY: u64 = 3;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    EXECUTION_MODE.save(deps.storage, &0u64)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::RequestLoan {
            flashloans_contract,
            execution_mode,
            amount,
        } => execute_request_loan(deps, flashloans_contract, execution_mode, amount),
        ExecuteMsg::ProcessLoan {
            return_address,
            loan_amount,
            fee,
        } => execute_process_loan(deps, return_address, loan_amount, fee),
    }
}

/// Makes the neutron-flashloans-user contract request a loan. Allows to
/// specify the execution mode (how the contract must behave while handling
/// the ProcessLoan message, see msg.rs), which is required for the integration
/// tests. Also allows to specify the loan amount.
pub fn execute_request_loan(
    deps: DepsMut,
    flashloans_contract: Addr,
    execution_mode: u64,
    amount: Vec<Coin>,
) -> Result<Response<NeutronMsg>, ContractError> {
    EXECUTION_MODE.save(deps.storage, &execution_mode)?;
    // Saving the flashloans contract address is necessary for the
    // MODE_REQUEST_ANOTHER_LOAN_RECURSIVELY execution mode.
    FLASHLOANS_CONTRACT.save(deps.storage, &flashloans_contract)?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: flashloans_contract.to_string(),
        msg: to_json_binary(&RequestLoan { amount }).unwrap(),
        funds: vec![],
    });
    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_request_loan"))
}

pub fn execute_process_loan(
    deps: DepsMut,
    return_address: Addr,
    loan_amount: Vec<Coin>,
    fee: Vec<Coin>,
) -> Result<Response<NeutronMsg>, ContractError> {
    let execution_mode = EXECUTION_MODE.load(deps.storage)?;

    match execution_mode {
        // Return the correct amount
        MODE_RETURN_LOAN => {
            let mut return_amount: Vec<Coin> = vec![];
            for (idx, coin) in loan_amount.iter().enumerate() {
                return_amount.push(Coin::new(
                    (coin.amount + fee[idx].amount).u128(),
                    coin.denom.clone(),
                ))
            }

            let msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: return_address.to_string(),
                amount: return_amount,
            });

            Ok(Response::new()
                .add_message(msg)
                .add_attribute("action", "execute_process_loan_MODE_RETURN_LOAN"))
        }
        // Do not return the loan
        MODE_WITHHOLD_LOAN => {
            Ok(Response::new().add_attribute("action", "execute_process_loan_MODE_WITHHOLD_LOAN"))
        }
        // Return more that necessary
        MODE_RETURN_LOAN_MORE_THAN_NECESSARY => {
            let mut return_amount: Vec<Coin> = vec![];
            for (idx, coin) in loan_amount.iter().enumerate() {
                return_amount.push(Coin::new(
                    // Simply add 1
                    (coin.amount + fee[idx].amount + Uint128::one()).u128(),
                    coin.denom.clone(),
                ))
            }

            let msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: return_address.to_string(),
                amount: return_amount,
            });

            Ok(Response::new().add_message(msg).add_attribute(
                "action",
                "execute_process_loan_MODE_RETURN_LOAN_MORE_THAN_NECESSARY",
            ))
        }
        // Request another loan while processing the existing loan
        MODE_REQUEST_ANOTHER_LOAN_RECURSIVELY => {
            let flashloans_contract = FLASHLOANS_CONTRACT.load(deps.storage)?;
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: flashloans_contract.to_string(),
                msg: to_json_binary(&RequestLoan {
                    amount: vec![Coin::new(100u128, "untrn")],
                })
                .unwrap(),
                funds: vec![],
            });
            Ok(Response::new().add_message(msg).add_attribute(
                "action",
                "execute_request_loan_MODE_REQUEST_ANOTHER_LOAN_RECURSIVELY",
            ))
        }
        _ => Err(ContractError::Std(StdError::generic_err(
            "The ProcessLoan handler failed",
        ))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
