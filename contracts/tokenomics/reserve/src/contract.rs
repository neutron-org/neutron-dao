use crate::distribution_params::DistributionParams;
use crate::error::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use exec_control::pause::{
    can_pause, can_unpause, validate_duration, PauseError, PauseInfoResponse,
};
use neutron_sdk::bindings::query::NeutronQuery;

use crate::msg::{DistributeMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StatsResponse};
use crate::state::{
    Config, ATOM_BALANCE_BEFORE_WITHDRAWAL, ATOM_DENOM, CONFIG, LAST_BURNED_COINS_AMOUNT,
    LAST_DISTRIBUTION_TIME, NTRN_ATOM_CL_PAIR_ADDRESS, NTRN_BALANCE_BEFORE_WITHDRAWAL,
    NTRN_USDC_CL_PAIR_ADDRESS, PAUSED_UNTIL, TOTAL_DISTRIBUTED, TOTAL_RESERVED,
    USDC_BALANCE_BEFORE_WITHDRAWAL, USDC_DENOM,
};
use crate::vesting::{
    get_burned_coins, safe_burned_coins_for_period, update_distribution_stats, vesting_function,
};
use astroport::asset::{native_asset, PairInfo};
use astroport::pair::{
    Cw20HookMsg as PairCw20HookMsg, ExecuteMsg as PairExecuteMsg, QueryMsg as PairQueryMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};

/// A `reply` call code ID used for withdraw NTRN/ATOM liquidity sub-messages during migration.
const WITHDRAW_NTRN_ATOM_LIQUIDITY_REPLY_ID: u64 = 1;
/// A `reply` call code ID used for withdraw NTRN/USDC liquidity sub-messages during migration.
const WITHDRAW_NTRN_USDC_LIQUIDITY_REPLY_ID: u64 = 2;
/// A `reply` call code ID used for balance check after providing liquidity to a CL pool.
const POST_PROVIDE_LIQUIDITY_BALANCE_CHECK_REPLY_ID: u64 = 3;

//--------------------------------------------------------------------------------------------------
// Instantiation
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        denom: msg.denom,
        min_period: msg.min_period,
        distribution_contract: deps.api.addr_validate(msg.distribution_contract.as_str())?,
        treasury_contract: deps.api.addr_validate(msg.treasury_contract.as_str())?,
        distribution_rate: msg.distribution_rate,
        main_dao_address: deps.api.addr_validate(&msg.main_dao_address)?,
        security_dao_address: deps.api.addr_validate(&msg.security_dao_address)?,
        vesting_denominator: msg.vesting_denominator,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    TOTAL_DISTRIBUTED.save(deps.storage, &Uint128::zero())?;
    TOTAL_RESERVED.save(deps.storage, &Uint128::zero())?;
    LAST_DISTRIBUTION_TIME.save(deps.storage, &0)?;
    PAUSED_UNTIL.save(deps.storage, &None)?;
    LAST_BURNED_COINS_AMOUNT.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new())
}

pub fn execute_pause(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    sender: Addr,
    duration: u64,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    can_pause(
        &sender,
        &config.main_dao_address,
        &config.security_dao_address,
    )?;
    validate_duration(duration)?;

    let paused_until_height: u64 = env.block.height + duration;

    let already_paused_until = PAUSED_UNTIL.load(deps.storage)?;
    if already_paused_until.unwrap_or(0u64) >= paused_until_height {
        return Err(ContractError::PauseError(PauseError::InvalidDuration(
            "contracts are already paused for a greater or equal duration".to_string(),
        )));
    }

    PAUSED_UNTIL.save(deps.storage, &Some(paused_until_height))?;

    Ok(Response::new()
        .add_attribute("action", "execute_pause")
        .add_attribute("sender", sender)
        .add_attribute("paused_until_height", paused_until_height.to_string()))
}

pub fn execute_unpause(
    deps: DepsMut<NeutronQuery>,
    sender: Addr,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    can_unpause(&sender, &config.main_dao_address)?;

    PAUSED_UNTIL.save(deps.storage, &None)?;

    Ok(Response::new()
        .add_attribute("action", "execute_unpause")
        .add_attribute("sender", sender))
}

fn get_pause_info(deps: Deps<NeutronQuery>, env: &Env) -> StdResult<PauseInfoResponse> {
    Ok(match PAUSED_UNTIL.may_load(deps.storage)?.unwrap_or(None) {
        Some(paused_until_height) => {
            if env.block.height.ge(&paused_until_height) {
                PauseInfoResponse::Unpaused {}
            } else {
                PauseInfoResponse::Paused {
                    until_height: paused_until_height,
                }
            }
        }
        None => PauseInfoResponse::Unpaused {},
    })
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match get_pause_info(deps.as_ref(), &env)? {
        PauseInfoResponse::Paused { .. } => {
            return match msg {
                ExecuteMsg::Pause { duration } => execute_pause(deps, env, info.sender, duration),
                ExecuteMsg::Unpause {} => execute_unpause(deps, info.sender),
                _ => Err(ContractError::PauseError(PauseError::Paused {})),
            };
        }
        PauseInfoResponse::Unpaused {} => (),
    }

    match msg {
        // permissioned - owner
        ExecuteMsg::TransferOwnership(new_owner) => {
            execute_transfer_ownership(deps, info, api.addr_validate(&new_owner)?)
        }
        // permissionless
        ExecuteMsg::Distribute {} => execute_distribute(deps, env),

        // permissioned - owner
        ExecuteMsg::UpdateConfig {
            distribution_rate,
            min_period,
            distribution_contract,
            treasury_contract,
            security_dao_address,
            vesting_denominator,
        } => execute_update_config(
            deps,
            info,
            distribution_contract,
            treasury_contract,
            security_dao_address,
            DistributionParams {
                distribution_rate,
                min_period,
                vesting_denominator,
            },
        ),
        ExecuteMsg::Pause { duration } => execute_pause(deps, env, info.sender, duration),
        ExecuteMsg::Unpause {} => execute_unpause(deps, info.sender),
    }
}

pub fn execute_transfer_ownership(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_owner_addr: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sender_addr = info.sender;
    let old_owner = config.main_dao_address;
    if sender_addr != old_owner {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.main_dao_address = new_owner_addr.clone();
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "neutron/reserve/transfer_ownership")
        .add_attribute("previous_owner", old_owner)
        .add_attribute("new_owner", new_owner_addr))
}

pub fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    distribution_contract: Option<String>,
    treasury_contract: Option<String>,
    security_dao_address: Option<String>,
    distribution_params: DistributionParams,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.main_dao_address {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(min_period) = distribution_params.min_period {
        config.min_period = min_period;
    }
    if let Some(distribution_contract) = distribution_contract {
        config.distribution_contract = deps.api.addr_validate(distribution_contract.as_str())?;
    }
    if let Some(reserve_contract) = treasury_contract {
        config.treasury_contract = deps.api.addr_validate(reserve_contract.as_str())?;
    }
    if let Some(security_dao_address) = security_dao_address {
        config.security_dao_address = deps.api.addr_validate(security_dao_address.as_str())?;
    }
    if let Some(distribution_rate) = distribution_params.distribution_rate {
        config.distribution_rate = distribution_rate;
    }
    if let Some(vesting_denominator) = distribution_params.vesting_denominator {
        config.vesting_denominator = vesting_denominator;
    }

    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "neutron/reserve/update_config")
        .add_attribute("denom", config.denom)
        .add_attribute("min_period", config.min_period.to_string())
        .add_attribute("distribution_contract", config.distribution_contract)
        .add_attribute("distribution_rate", config.distribution_rate.to_string())
        .add_attribute(
            "vesting_denominator",
            config.vesting_denominator.to_string(),
        )
        .add_attribute("owner", config.main_dao_address))
}

pub fn execute_distribute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom.clone();
    let current_time = env.block.time.seconds();
    if current_time - LAST_DISTRIBUTION_TIME.load(deps.storage)? < config.min_period {
        return Err(ContractError::TooSoonToDistribute {});
    }
    LAST_DISTRIBUTION_TIME.save(deps.storage, &current_time)?;
    let current_balance = deps
        .querier
        .query_balance(env.contract.address, &denom)?
        .amount;

    if current_balance.is_zero() {
        return Err(ContractError::NoFundsToDistribute {});
    }

    let last_burned_coins = LAST_BURNED_COINS_AMOUNT.load(deps.storage)?;
    let burned_coins = get_burned_coins(deps.as_ref(), &denom)?;

    let burned_coins_for_period = safe_burned_coins_for_period(burned_coins, last_burned_coins)?;

    if burned_coins_for_period == 0 {
        return Err(ContractError::NoBurnedCoins {});
    }

    let balance_to_distribute = vesting_function(
        current_balance,
        burned_coins_for_period,
        config.vesting_denominator,
    )?;

    let to_distribute = balance_to_distribute * config.distribution_rate;

    let to_reserve = balance_to_distribute.checked_sub(to_distribute)?;

    update_distribution_stats(
        deps,
        to_distribute,
        to_reserve,
        last_burned_coins.checked_add(Uint128::from(burned_coins_for_period))?,
    )?;
    let resp = create_distribution_response(config, to_distribute, to_reserve, denom)?;

    Ok(resp
        .add_attribute("action", "neutron/reserve/distribute")
        .add_attribute("reserve", to_reserve)
        .add_attribute("distributed", to_distribute))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Stats {} => to_binary(&query_stats(deps)?),
        QueryMsg::PauseInfo {} => query_paused(deps, env),
    }
}

pub fn query_paused(deps: Deps<NeutronQuery>, env: Env) -> StdResult<Binary> {
    to_binary(&get_pause_info(deps, &env)?)
}

pub fn query_config(deps: Deps<NeutronQuery>) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_stats(deps: Deps<NeutronQuery>) -> StdResult<StatsResponse> {
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.storage)?;
    let total_reserved = TOTAL_RESERVED.load(deps.storage)?;
    let total_processed_burned_coins = LAST_BURNED_COINS_AMOUNT.load(deps.storage)?;

    Ok(StatsResponse {
        total_distributed,
        total_reserved,
        total_processed_burned_coins,
    })
}

//--------------------------------------------------------------------------------------------------
// Helpers
//--------------------------------------------------------------------------------------------------

pub fn create_distribution_response(
    config: Config,
    to_distribute: Uint128,
    to_reserve: Uint128,
    denom: String,
) -> StdResult<Response> {
    let mut resp = Response::default();
    if !to_distribute.is_zero() {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.distribution_contract.to_string(),
            funds: coins(to_distribute.u128(), denom.clone()),
            msg: to_binary(&DistributeMsg::Fund {})?,
        });
        resp = resp.add_message(msg)
    }

    if !to_reserve.is_zero() {
        let msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: config.treasury_contract.to_string(),
            amount: coins(to_reserve.u128(), denom),
        });
        resp = resp.add_message(msg);
    }

    Ok(resp)
}

/// Withdraws liquidity from Astroport xyk pools and provides it to the concentrated liquidity ones.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let mut resp: Response = Response::default();

    NTRN_ATOM_CL_PAIR_ADDRESS.save(deps.storage, &msg.ntrn_atom_cl_pair_address)?;
    NTRN_USDC_CL_PAIR_ADDRESS.save(deps.storage, &msg.ntrn_usdc_cl_pair_address)?;
    ATOM_DENOM.save(deps.storage, &msg.atom_denom.clone())?;
    USDC_DENOM.save(deps.storage, &msg.usdc_denom.clone())?;

    NTRN_BALANCE_BEFORE_WITHDRAWAL.save(
        deps.storage,
        &deps
            .querier
            .query_balance(env.contract.address.to_string(), "untrn")?
            .amount,
    )?;
    ATOM_BALANCE_BEFORE_WITHDRAWAL.save(
        deps.storage,
        &deps
            .querier
            .query_balance(env.contract.address.to_string(), msg.atom_denom)?
            .amount,
    )?;
    USDC_BALANCE_BEFORE_WITHDRAWAL.save(
        deps.storage,
        &deps
            .querier
            .query_balance(env.contract.address.to_string(), msg.usdc_denom)?
            .amount,
    )?;

    // get the NTRN/ATOM pair LP token address and contract's balance
    let ntrn_atom_xyk_pair_info: PairInfo = deps.querier.query_wasm_smart(
        msg.ntrn_atom_xyk_pair_address.clone(),
        &to_binary(&PairQueryMsg::Pair {})?,
    )?;
    let ntrn_atom_xyk_pair_lp_token_share: Uint128 = deps.querier.query_wasm_smart(
        ntrn_atom_xyk_pair_info.liquidity_token.clone(),
        &Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    // get the NTRN/USDC pair LP token address and contract's balance
    let ntrn_usdc_xyk_pair_info: PairInfo = deps.querier.query_wasm_smart(
        msg.ntrn_usdc_xyk_pair_address.clone(),
        &to_binary(&PairQueryMsg::Pair {})?,
    )?;
    let ntrn_usdc_xyk_pair_lp_token_share: Uint128 = deps.querier.query_wasm_smart(
        ntrn_usdc_xyk_pair_info.liquidity_token.clone(),
        &Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    // register submessages for liquidity withdrawal from both pools
    resp = resp.add_submessages(vec![
        SubMsg {
            id: WITHDRAW_NTRN_ATOM_LIQUIDITY_REPLY_ID,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: ntrn_atom_xyk_pair_info.liquidity_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: msg.ntrn_atom_xyk_pair_address,
                    amount: ntrn_atom_xyk_pair_lp_token_share,
                    msg: to_binary(&PairCw20HookMsg::WithdrawLiquidity { assets: vec![] })?,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success,
        },
        SubMsg {
            id: WITHDRAW_NTRN_USDC_LIQUIDITY_REPLY_ID,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: ntrn_usdc_xyk_pair_info.liquidity_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: msg.ntrn_usdc_xyk_pair_address,
                    amount: ntrn_usdc_xyk_pair_lp_token_share,
                    msg: to_binary(&PairCw20HookMsg::WithdrawLiquidity { assets: vec![] })?,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success,
        },
    ]);

    Ok(resp)
}

/// The entry point to the contract for processing replies from submessages.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut resp = Response::default();
    match msg.id {
        WITHDRAW_NTRN_ATOM_LIQUIDITY_REPLY_ID => {
            let atom_denom = ATOM_DENOM.load(deps.storage)?;
            let ntrn_balance_after_withdrawal = deps
                .querier
                .query_balance(env.contract.address.to_string(), "untrn")?
                .amount;
            let atom_balance_after_withdrawal = deps
                .querier
                .query_balance(env.contract.address.to_string(), atom_denom.clone())?
                .amount;

            // calc amount of assets that's been withdrawn
            let withdrawn_ntrn_amount = NTRN_BALANCE_BEFORE_WITHDRAWAL
                .load(deps.storage)?
                .checked_sub(ntrn_balance_after_withdrawal)?;
            let withdrawn_atom_amount = ATOM_BALANCE_BEFORE_WITHDRAWAL
                .load(deps.storage)?
                .checked_sub(atom_balance_after_withdrawal)?;

            // provide the withdrawn assets to the CL pair
            resp = resp.add_submessage(SubMsg {
                id: POST_PROVIDE_LIQUIDITY_BALANCE_CHECK_REPLY_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NTRN_ATOM_CL_PAIR_ADDRESS.load(deps.storage)?,
                    msg: to_binary(&PairExecuteMsg::ProvideLiquidity {
                        assets: vec![
                            native_asset(String::from("untrn"), withdrawn_ntrn_amount),
                            native_asset(atom_denom.clone(), withdrawn_atom_amount),
                        ],
                        slippage_tolerance: None,
                        auto_stake: None,
                        receiver: None,
                    })?,
                    funds: vec![
                        Coin::new(withdrawn_ntrn_amount.into(), String::from("untrn")),
                        Coin::new(withdrawn_atom_amount.into(), atom_denom),
                    ],
                }),
                gas_limit: None,
                reply_on: cosmwasm_std::ReplyOn::Never,
            });
        }

        WITHDRAW_NTRN_USDC_LIQUIDITY_REPLY_ID => {
            let usdc_denom = USDC_DENOM.load(deps.storage)?;
            let ntrn_balance_after_withdrawal = deps
                .querier
                .query_balance(env.contract.address.to_string(), "untrn")?
                .amount;
            let usdc_balance_after_withdrawal = deps
                .querier
                .query_balance(env.contract.address.to_string(), usdc_denom.clone())?
                .amount;

            // calc amount of assets that's been withdrawn
            let withdrawn_ntrn_amount = NTRN_BALANCE_BEFORE_WITHDRAWAL
                .load(deps.storage)?
                .checked_sub(ntrn_balance_after_withdrawal)?;
            let withdrawn_usdc_amount = USDC_BALANCE_BEFORE_WITHDRAWAL
                .load(deps.storage)?
                .checked_sub(usdc_balance_after_withdrawal)?;

            // provide the withdrawn assets to the CL pair
            resp = resp.add_submessage(SubMsg {
                id: POST_PROVIDE_LIQUIDITY_BALANCE_CHECK_REPLY_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NTRN_USDC_CL_PAIR_ADDRESS.load(deps.storage)?,
                    msg: to_binary(&PairExecuteMsg::ProvideLiquidity {
                        assets: vec![
                            native_asset(String::from("untrn"), withdrawn_ntrn_amount),
                            native_asset(usdc_denom.clone(), withdrawn_usdc_amount),
                        ],
                        slippage_tolerance: None,
                        auto_stake: None,
                        receiver: None,
                    })?,
                    funds: vec![
                        Coin::new(withdrawn_ntrn_amount.into(), String::from("untrn")),
                        Coin::new(withdrawn_usdc_amount.into(), usdc_denom),
                    ],
                }),
                gas_limit: None,
                reply_on: cosmwasm_std::ReplyOn::Never,
            });
        }

        POST_PROVIDE_LIQUIDITY_BALANCE_CHECK_REPLY_ID => {
            let atom_denom = ATOM_DENOM.load(deps.storage)?;
            let usdc_denom = USDC_DENOM.load(deps.storage)?;

            let ntrn_balance = deps
                .querier
                .query_balance(env.contract.address.to_string(), "untrn")?
                .amount;
            let atom_balance = deps
                .querier
                .query_balance(env.contract.address.to_string(), atom_denom.clone())?
                .amount;
            let usdc_balance = deps
                .querier
                .query_balance(env.contract.address.to_string(), usdc_denom.clone())?
                .amount;

            let initial_ntrn_amount = NTRN_BALANCE_BEFORE_WITHDRAWAL.load(deps.storage)?;
            let initial_atom_amount = ATOM_BALANCE_BEFORE_WITHDRAWAL.load(deps.storage)?;
            let initial_usdc_amount = USDC_BALANCE_BEFORE_WITHDRAWAL.load(deps.storage)?;

            if !ntrn_balance.eq(&initial_ntrn_amount) {
                return Err(ContractError::MigrationBalancesMismtach {
                    denom: String::from("untrn"),
                    initial_balance: initial_ntrn_amount,
                    intermediate_balance: ntrn_balance,
                });
            }
            if !atom_balance.eq(&initial_atom_amount) {
                return Err(ContractError::MigrationBalancesMismtach {
                    denom: atom_denom,
                    initial_balance: initial_atom_amount,
                    intermediate_balance: atom_balance,
                });
            }
            if !usdc_balance.eq(&initial_usdc_amount) {
                return Err(ContractError::MigrationBalancesMismtach {
                    denom: usdc_denom,
                    initial_balance: initial_usdc_amount,
                    intermediate_balance: usdc_balance,
                });
            }
        }
        _ => return Err(ContractError::UnkownReplyID { reply_id: msg.id }),
    }
    Ok(resp)
}
