use crate::error::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, WasmMsg,
};
use exec_control::pause::{
    can_pause, can_unpause, validate_duration, PauseError, PauseInfoResponse,
};

use crate::msg::{DistributeMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StatsResponse};
use crate::state::{
    Config, CONFIG, LAST_DISTRIBUTION_TIME, PAUSED_UNTIL, TOTAL_DISTRIBUTED, TOTAL_RECEIVED,
    TOTAL_RESERVED,
};

//--------------------------------------------------------------------------------------------------
// Instantiation
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        denom: msg.denom,
        min_period: msg.min_period,
        distribution_contract: deps.api.addr_validate(msg.distribution_contract.as_str())?,
        reserve_contract: deps.api.addr_validate(msg.reserve_contract.as_str())?,
        distribution_rate: msg.distribution_rate,
        main_dao_address: deps.api.addr_validate(&msg.main_dao_address)?,
        security_dao_address: deps.api.addr_validate(&msg.security_dao_address)?,
    };
    CONFIG.save(deps.storage, &config)?;
    TOTAL_RECEIVED.save(deps.storage, &Uint128::zero())?;
    TOTAL_DISTRIBUTED.save(deps.storage, &Uint128::zero())?;
    TOTAL_RESERVED.save(deps.storage, &Uint128::zero())?;
    LAST_DISTRIBUTION_TIME.save(deps.storage, &0)?;
    PAUSED_UNTIL.save(deps.storage, &None)?;

    Ok(Response::new())
}

pub fn execute_pause(
    deps: DepsMut,
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

pub fn execute_unpause(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    can_unpause(&sender, &config.main_dao_address)?;

    PAUSED_UNTIL.save(deps.storage, &None)?;

    Ok(Response::new()
        .add_attribute("action", "execute_unpause")
        .add_attribute("sender", sender))
}

fn get_pause_info(deps: Deps, env: &Env) -> StdResult<PauseInfoResponse> {
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
    deps: DepsMut,
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
            reserve_contract,
            security_dao_address,
        } => execute_update_config(
            deps,
            info,
            distribution_rate,
            min_period,
            distribution_contract,
            reserve_contract,
            security_dao_address,
        ),
        ExecuteMsg::Pause { duration } => execute_pause(deps, env, info.sender, duration),
        ExecuteMsg::Unpause {} => execute_unpause(deps, info.sender),
    }
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
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
        .add_attribute("action", "neutron/treasury/transfer_ownership")
        .add_attribute("previous_owner", old_owner)
        .add_attribute("new_owner", new_owner_addr))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    distribution_rate: Option<Decimal>,
    min_period: Option<u64>,
    distribution_contract: Option<String>,
    reserve_contract: Option<String>,
    security_dao_address: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.main_dao_address {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(min_period) = min_period {
        config.min_period = min_period;
    }
    if let Some(distribution_contract) = distribution_contract {
        config.distribution_contract = deps.api.addr_validate(distribution_contract.as_str())?;
    }
    if let Some(reserve_contract) = reserve_contract {
        config.reserve_contract = deps.api.addr_validate(reserve_contract.as_str())?;
    }
    if let Some(security_dao_address) = security_dao_address {
        config.security_dao_address = deps.api.addr_validate(security_dao_address.as_str())?;
    }
    if let Some(distribution_rate) = distribution_rate {
        if (distribution_rate > Decimal::one()) || (distribution_rate < Decimal::zero()) {
            return Err(ContractError::InvalidDistributionRate(
                "distribution_rate must be between 0 and 1".to_string(),
            ));
        }
        config.distribution_rate = distribution_rate;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "neutron/treasury/update_config")
        .add_attribute("denom", config.denom)
        .add_attribute("min_period", config.min_period.to_string())
        .add_attribute("distribution_contract", config.distribution_contract)
        .add_attribute("distribution_rate", config.distribution_rate.to_string())
        .add_attribute("owner", config.main_dao_address))
}

pub fn execute_distribute(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
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

    let to_distribute = current_balance * config.distribution_rate;
    let to_reserve = current_balance.checked_sub(to_distribute)?;
    // update stats
    let total_received = TOTAL_RECEIVED.load(deps.storage)?;
    TOTAL_RECEIVED.save(
        deps.storage,
        &(total_received.checked_add(current_balance)?),
    )?;
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.storage)?;
    TOTAL_DISTRIBUTED.save(
        deps.storage,
        &(total_distributed.checked_add(to_distribute)?),
    )?;
    let total_reserved = TOTAL_RESERVED.load(deps.storage)?;
    TOTAL_RESERVED.save(deps.storage, &(total_reserved.checked_add(to_reserve)?))?;

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
            to_address: config.reserve_contract.to_string(),
            amount: coins(to_reserve.u128(), denom),
        });
        resp = resp.add_message(msg);
    }

    Ok(resp
        .add_attribute("action", "neutron/treasury/distribute")
        .add_attribute("reserved", to_reserve)
        .add_attribute("distributed", to_distribute))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Stats {} => to_binary(&query_stats(deps)?),
        QueryMsg::PauseInfo {} => query_paused(deps, env),
    }
}

pub fn query_paused(deps: Deps, env: Env) -> StdResult<Binary> {
    to_binary(&get_pause_info(deps, &env)?)
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_stats(deps: Deps) -> StdResult<StatsResponse> {
    let total_received = TOTAL_RECEIVED.load(deps.storage)?;
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.storage)?;
    let total_reserved = TOTAL_RESERVED.load(deps.storage)?;

    Ok(StatsResponse {
        total_received,
        total_distributed,
        total_reserved,
    })
}
