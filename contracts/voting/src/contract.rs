#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::helpers::{compute_position_response, compute_withdrawable};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PositionResponse, QueryMsg, Schedule,
    VotingPowerResponse, VEST_DENOM,
};
use crate::state::{Position, OWNER, POSITIONS, UNLOCK_SCHEDULE};

const CONTRACT_NAME: &str = "crates.io:mars-voting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

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
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    OWNER.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;
    UNLOCK_SCHEDULE.save(deps.storage, &msg.unlock_schedule)?;

    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let api = deps.api;
    match msg {
        ExecuteMsg::CreatePosition {
            user,
            vest_schedule,
        } => create_position(deps, info, api.addr_validate(&user)?, vest_schedule),
        ExecuteMsg::Withdraw {} => withdraw(deps, env.block.time.seconds(), info.sender),
        ExecuteMsg::TransferOwnership(new_owner) => {
            transfer_ownership(deps, info.sender, api.addr_validate(&new_owner)?)
        }
    }
}

pub fn create_position(
    deps: DepsMut,
    info: MessageInfo,
    user_addr: Addr,
    vest_schedule: Schedule,
) -> StdResult<Response> {
    // only owner can create allocations
    let owner_addr = OWNER.load(deps.storage)?;
    if info.sender != owner_addr {
        return Err(StdError::generic_err("only owner can create allocations"));
    }

    // must send exactly one coin
    if info.funds.len() != 1 {
        return Err(StdError::generic_err(format!(
            "wrong number of coins: expecting 1, received {}",
            info.funds.len()
        )));
    }

    // the coin must be the voting coin
    let coin = &info.funds[0];
    if coin.denom != VEST_DENOM {
        return Err(StdError::generic_err(format!(
            "wrong denom: expecting {}, received {}",
            VEST_DENOM, coin.denom
        )));
    }

    // the amount must be greater than zero
    let total = coin.amount;
    if total.is_zero() {
        return Err(StdError::generic_err("wrong amount: must be greater than zero"));
    }

    POSITIONS.update(deps.storage, &user_addr, |position| {
        if position.is_some() {
            return Err(StdError::generic_err("user has a voting position"));
        }
        Ok(Position {
            total,
            vest_schedule,
            withdrawn: Uint128::zero(),
        })
    })?;

    Ok(Response::new()
        .add_attribute("action", "mars/voting/position_created")
        .add_attribute("user", user_addr)
        .add_attribute("total", total)
        .add_attribute("start_time", vest_schedule.start_time.to_string())
        .add_attribute("cliff", vest_schedule.cliff.to_string())
        .add_attribute("duration", vest_schedule.duration.to_string()))
}

pub fn withdraw(deps: DepsMut, time: u64, user_addr: Addr) -> StdResult<Response> {
    let unlock_schedule = UNLOCK_SCHEDULE.load(deps.storage)?;
    let mut position = POSITIONS.load(deps.storage, &user_addr)?;

    let (_, _, withdrawable) = compute_withdrawable(
        time,
        position.total,
        position.withdrawn,
        position.vest_schedule,
        unlock_schedule,
    );

    if withdrawable.is_zero() {
        return Err(StdError::generic_err("withdrawable amount is zero"));
    }

    position.withdrawn += withdrawable;
    POSITIONS.save(deps.storage, &user_addr, &position)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: user_addr.to_string(),
            amount: coins(withdrawable.u128(), VEST_DENOM),
        }))
        .add_attribute("action", "mars/voting/withdraw")
        .add_attribute("user", user_addr)
        .add_attribute("timestamp", time.to_string())
        .add_attribute("withdrawable", withdrawable))
}

pub fn transfer_ownership(
    deps: DepsMut,
    sender_addr: Addr,
    new_owner_addr: Addr,
) -> StdResult<Response> {
    let owner_addr = OWNER.load(deps.storage)?;
    if sender_addr != owner_addr {
        return Err(StdError::generic_err("only owner can transfer ownership"));
    }

    OWNER.save(deps.storage, &new_owner_addr)?;

    Ok(Response::new()
        .add_attribute("action", "mars/voting/transfer_ownership")
        .add_attribute("previous_owner", owner_addr)
        .add_attribute("new_owner", new_owner_addr))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::VotingPower {
            user,
        } => to_binary(&query_voting_power(deps, api.addr_validate(&user)?)?),
        QueryMsg::VotingPowers {
            start_after,
            limit,
        } => to_binary(&query_voting_powers(deps, start_after, limit)?),
        QueryMsg::Position {
            user,
        } => to_binary(&query_position(deps, env.block.time.seconds(), api.addr_validate(&user)?)?),
        QueryMsg::Positions {
            start_after,
            limit,
        } => to_binary(&query_positions(deps, env.block.time.seconds(), start_after, limit)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        owner: OWNER.load(deps.storage)?.into(),
        unlock_schedule: UNLOCK_SCHEDULE.load(deps.storage)?,
    })
}

pub fn query_voting_power(deps: Deps, user_addr: Addr) -> StdResult<VotingPowerResponse> {
    let voting_power = match POSITIONS.may_load(deps.storage, &user_addr) {
        Ok(Some(position)) => position.total - position.withdrawn,
        Ok(None) => Uint128::zero(),
        Err(err) => return Err(err),
    };

    Ok(VotingPowerResponse {
        user: user_addr.to_string(),
        voting_power,
    })
}

pub fn query_position(deps: Deps, time: u64, user_addr: Addr) -> StdResult<PositionResponse> {
    let unlock_schedule = UNLOCK_SCHEDULE.load(deps.storage)?;
    let position = POSITIONS.load(deps.storage, &user_addr)?;

    Ok(compute_position_response(time, user_addr, &position, unlock_schedule))
}

pub fn query_voting_powers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<VotingPowerResponse>> {
    let addr: Addr;
    let start = match &start_after {
        Some(addr_str) => {
            addr = deps.api.addr_validate(addr_str)?;
            Some(Bound::exclusive(&addr))
        }
        None => None,
    };

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    POSITIONS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            let (user_addr, position) = res?;
            Ok(VotingPowerResponse {
                user: user_addr.to_string(),
                voting_power: position.total - position.withdrawn,
            })
        })
        .collect()
}

pub fn query_positions(
    deps: Deps,
    time: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<PositionResponse>> {
    let unlock_schedule = UNLOCK_SCHEDULE.load(deps.storage)?;

    let addr: Addr;
    let start = match &start_after {
        Some(addr_str) => {
            addr = deps.api.addr_validate(addr_str)?;
            Some(Bound::exclusive(&addr))
        }
        None => None,
    };

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    POSITIONS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            let (user_addr, position) = res?;
            Ok(compute_position_response(time, user_addr, &position, unlock_schedule))
        })
        .collect()
}
