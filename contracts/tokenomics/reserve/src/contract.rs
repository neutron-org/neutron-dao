use crate::distribution_params::DistributionParams;
use crate::error::ContractError;
use crate::msg::{
    CallbackMsg, DistributeMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StatsResponse,
};
use crate::state::{
    Config, XykToClMigrationConfig, CONFIG, LAST_BURNED_COINS_AMOUNT, LAST_DISTRIBUTION_TIME,
    PAUSED_UNTIL, TOTAL_DISTRIBUTED, TOTAL_RESERVED, XYK_TO_CL_MIGRATION_CONFIG,
};
use crate::vesting::{
    get_burned_coins, safe_burned_coins_for_period, update_distribution_stats, vesting_function,
};
use astroport::asset::{native_asset, PairInfo};
use astroport::pair::{
    Cw20HookMsg as PairCw20HookMsg, ExecuteMsg as PairExecuteMsg, QueryMsg as PairQueryMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use exec_control::pause::{
    can_pause, can_unpause, validate_duration, PauseError, PauseInfoResponse,
};
use neutron_sdk::bindings::query::NeutronQuery;

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-reserve";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        ExecuteMsg::MigrateFromXykToCl {
            slippage_tolerance,
            ntrn_atom_amount,
            ntrn_usdc_amount,
        } => execute_migrate_from_xyk_to_cl(
            deps,
            env,
            slippage_tolerance,
            ntrn_atom_amount,
            ntrn_usdc_amount,
        ),
        ExecuteMsg::Callback(msg) => _handle_callback(deps, env, info, msg),
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

fn execute_migrate_from_xyk_to_cl(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    slippage_tolerance: Option<Decimal>,
    ntrn_atom_amount: Option<Uint128>,
    ntrn_usdc_amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let migration_config: XykToClMigrationConfig = XYK_TO_CL_MIGRATION_CONFIG.load(deps.storage)?;

    // get pairs LP token addresses
    let ntrn_atom_pair_info: PairInfo = deps.querier.query_wasm_smart(
        migration_config.ntrn_atom_xyk_pair.clone(),
        &PairQueryMsg::Pair {},
    )?;
    let ntrn_usdc_pair_info: PairInfo = deps.querier.query_wasm_smart(
        migration_config.ntrn_usdc_xyk_pair.clone(),
        &PairQueryMsg::Pair {},
    )?;

    // query max available amounts to be withdrawn from both pairs
    let max_available_ntrn_atom_amount = {
        let resp: BalanceResponse = deps.querier.query_wasm_smart(
            ntrn_atom_pair_info.liquidity_token.clone(),
            &Cw20QueryMsg::Balance {
                address: env.contract.address.to_string(),
            },
        )?;
        resp.balance
    };
    let max_available_ntrn_usdc_amount = {
        let resp: BalanceResponse = deps.querier.query_wasm_smart(
            ntrn_usdc_pair_info.liquidity_token.clone(),
            &Cw20QueryMsg::Balance {
                address: env.contract.address.to_string(),
            },
        )?;
        resp.balance
    };
    if max_available_ntrn_atom_amount.is_zero() && max_available_ntrn_usdc_amount.is_zero() {
        return Err(ContractError::MigrationComplete {});
    }

    let ntrn_atom_amount = ntrn_atom_amount.unwrap_or(max_available_ntrn_atom_amount);
    let ntrn_usdc_amount = ntrn_usdc_amount.unwrap_or(max_available_ntrn_usdc_amount);
    let slippage_tolerance = slippage_tolerance.unwrap_or(migration_config.max_slippage);

    // validate parameters to the max available values
    if ntrn_atom_amount.gt(&max_available_ntrn_atom_amount) {
        return Err(ContractError::MigrationAmountUnavailable {
            amount: ntrn_atom_amount,
            max_amount: max_available_ntrn_atom_amount,
        });
    }
    if ntrn_usdc_amount.gt(&max_available_ntrn_usdc_amount) {
        return Err(ContractError::MigrationAmountUnavailable {
            amount: ntrn_usdc_amount,
            max_amount: max_available_ntrn_usdc_amount,
        });
    }
    if slippage_tolerance.gt(&migration_config.max_slippage) {
        return Err(ContractError::MigrationSlippageToBig {
            slippage_tolerance,
            max_slippage_tolerance: migration_config.max_slippage,
        });
    }

    let mut resp = Response::default();
    if !ntrn_atom_amount.is_zero() {
        resp = resp.add_message(
            CallbackMsg::MigrateLiquidityToClPair {
                ntrn_denom: migration_config.ntrn_denom.clone(),
                amount: ntrn_atom_amount,
                slippage_tolerance,
                xyk_pair: migration_config.ntrn_atom_xyk_pair,
                xyk_lp_token: ntrn_atom_pair_info.liquidity_token,
                cl_pair: migration_config.ntrn_atom_cl_pair,
                paired_asset_denom: migration_config.atom_denom,
            }
            .to_cosmos_msg(&env)?,
        );
    }
    if !ntrn_usdc_amount.is_zero() {
        resp = resp.add_message(
            CallbackMsg::MigrateLiquidityToClPair {
                ntrn_denom: migration_config.ntrn_denom,
                amount: ntrn_usdc_amount,
                slippage_tolerance,
                xyk_pair: migration_config.ntrn_usdc_xyk_pair,
                xyk_lp_token: ntrn_usdc_pair_info.liquidity_token,
                cl_pair: migration_config.ntrn_usdc_cl_pair,
                paired_asset_denom: migration_config.usdc_denom,
            }
            .to_cosmos_msg(&env)?,
        );
    }

    Ok(resp)
}

fn _handle_callback(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: CallbackMsg,
) -> Result<Response, ContractError> {
    // Only the contract itself can call callbacks
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        CallbackMsg::MigrateLiquidityToClPair {
            xyk_pair,
            xyk_lp_token,
            amount,
            slippage_tolerance,
            cl_pair,
            ntrn_denom,
            paired_asset_denom,
        } => migrate_liquidity_to_cl_pair_callback(
            deps,
            env,
            xyk_pair,
            xyk_lp_token,
            amount,
            slippage_tolerance,
            cl_pair,
            ntrn_denom,
            paired_asset_denom,
        ),
        CallbackMsg::ProvideLiquidityToClPairAfterWithdrawal {
            ntrn_denom,
            ntrn_init_balance,
            paired_asset_denom,
            paired_asset_init_balance,
            cl_pair: cl_pair_address,
            slippage_tolerance,
        } => provide_liquidity_to_cl_pair_after_withdrawal_callback(
            deps,
            env,
            ntrn_denom,
            ntrn_init_balance,
            paired_asset_denom,
            paired_asset_init_balance,
            cl_pair_address,
            slippage_tolerance,
        ),
        CallbackMsg::PostMigrationBalancesCheck {
            ntrn_denom,
            ntrn_init_balance,
            paired_asset_denom,
            paired_asset_init_balance,
        } => post_migration_balances_check_callback(
            deps,
            env,
            ntrn_denom,
            ntrn_init_balance,
            paired_asset_denom,
            paired_asset_init_balance,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn migrate_liquidity_to_cl_pair_callback(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    xyk_pair: Addr,
    xyk_lp_token: Addr,
    amount: Uint128,
    slippage_tolerance: Decimal,
    cl_pair: Addr,
    ntrn_denom: String,
    paired_asset_denom: String,
) -> Result<Response, ContractError> {
    let ntrn_init_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), ntrn_denom.clone())?
        .amount;
    let paired_asset_init_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), paired_asset_denom.clone())?
        .amount;

    let msgs: Vec<CosmosMsg> = vec![
        // push message to withdraw liquidity from the xyk pair
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: xyk_lp_token.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: xyk_pair.to_string(),
                amount,
                msg: to_json_binary(&PairCw20HookMsg::WithdrawLiquidity { assets: vec![] })?,
            })?,
            funds: vec![],
        }),
        // push the next migration step as a callback message
        CallbackMsg::ProvideLiquidityToClPairAfterWithdrawal {
            ntrn_denom,
            ntrn_init_balance,
            paired_asset_denom,
            paired_asset_init_balance,
            cl_pair,
            slippage_tolerance,
        }
        .to_cosmos_msg(&env)?,
    ];

    Ok(Response::default().add_messages(msgs))
}

#[allow(clippy::too_many_arguments)]
fn provide_liquidity_to_cl_pair_after_withdrawal_callback(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    ntrn_denom: String,
    ntrn_init_balance: Uint128,
    paired_asset_denom: String,
    paired_asset_init_balance: Uint128,
    cl_pair_address: Addr,
    slippage_tolerance: Decimal,
) -> Result<Response, ContractError> {
    let ntrn_balance_after_withdrawal = deps
        .querier
        .query_balance(env.contract.address.to_string(), ntrn_denom.clone())?
        .amount;
    let paired_asset_balance_after_withdrawal = deps
        .querier
        .query_balance(env.contract.address.to_string(), paired_asset_denom.clone())?
        .amount;

    // calc amount of assets that's been withdrawn
    let withdrawn_ntrn_amount = ntrn_balance_after_withdrawal.checked_sub(ntrn_init_balance)?;
    let withdrawn_paired_asset_amount =
        paired_asset_balance_after_withdrawal.checked_sub(paired_asset_init_balance)?;

    let config = CONFIG.load(deps.storage)?;

    let msgs: Vec<CosmosMsg> = vec![
        // push message to provide liquidity to the CL pair
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cl_pair_address.to_string(),
            msg: to_json_binary(&PairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    native_asset(ntrn_denom.clone(), withdrawn_ntrn_amount),
                    native_asset(paired_asset_denom.clone(), withdrawn_paired_asset_amount),
                ],
                slippage_tolerance: Some(slippage_tolerance),
                auto_stake: None,
                receiver: Option::from(config.main_dao_address.to_string()),
            })?,
            funds: vec![
                Coin::new(withdrawn_ntrn_amount.into(), ntrn_denom.clone()),
                Coin::new(
                    withdrawn_paired_asset_amount.into(),
                    paired_asset_denom.clone(),
                ),
            ],
        }),
        // push the next migration step as a callback message
        CallbackMsg::PostMigrationBalancesCheck {
            ntrn_denom,
            ntrn_init_balance,
            paired_asset_denom,
            paired_asset_init_balance,
        }
        .to_cosmos_msg(&env)?,
    ];

    Ok(Response::default().add_messages(msgs))
}

fn post_migration_balances_check_callback(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    ntrn_denom: String,
    ntrn_init_balance: Uint128,
    paired_asset_denom: String,
    paired_asset_init_balance: Uint128,
) -> Result<Response, ContractError> {
    let ntrn_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), ntrn_denom.clone())?
        .amount;
    let paired_asset_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), paired_asset_denom.clone())?
        .amount;

    if !ntrn_balance.eq(&ntrn_init_balance) {
        return Err(ContractError::MigrationBalancesMismatch {
            denom: ntrn_denom,
            initial_balance: ntrn_init_balance,
            final_balance: ntrn_balance,
        });
    }
    if !paired_asset_balance.eq(&paired_asset_init_balance) {
        return Err(ContractError::MigrationBalancesMismatch {
            denom: paired_asset_denom,
            initial_balance: paired_asset_init_balance,
            final_balance: paired_asset_balance,
        });
    }

    Ok(Response::default())
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Stats {} => to_json_binary(&query_stats(deps)?),
        QueryMsg::PauseInfo {} => query_paused(deps, env),
    }
}

pub fn query_paused(deps: Deps<NeutronQuery>, env: Env) -> StdResult<Binary> {
    to_json_binary(&get_pause_info(deps, &env)?)
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
            msg: to_json_binary(&DistributeMsg::Fund {})?,
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

//--------------------------------------------------------------------------------------------------
// Migration
//--------------------------------------------------------------------------------------------------

/// Withdraws liquidity from Astroport xyk pairs and provides it to the concentrated liquidity ones.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    XYK_TO_CL_MIGRATION_CONFIG.save(
        deps.storage,
        &XykToClMigrationConfig {
            max_slippage: msg.max_slippage,
            ntrn_denom: msg.ntrn_denom,
            atom_denom: msg.atom_denom,
            ntrn_atom_xyk_pair: deps.api.addr_validate(msg.ntrn_atom_xyk_pair.as_str())?,
            ntrn_atom_cl_pair: deps.api.addr_validate(msg.ntrn_atom_cl_pair.as_str())?,
            usdc_denom: msg.usdc_denom,
            ntrn_usdc_xyk_pair: deps.api.addr_validate(msg.ntrn_usdc_xyk_pair.as_str())?,
            ntrn_usdc_cl_pair: deps.api.addr_validate(msg.ntrn_usdc_cl_pair.as_str())?,
        },
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
