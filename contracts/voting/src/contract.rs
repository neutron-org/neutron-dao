#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, VotingPowerResponse},
    state::{DENOM, OWNER, TOKENS_LOCKED},
    types::{ContractError, ContractResult},
};
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use std::cmp::Ordering;

const CONTRACT_NAME: &str = "crates.io:neutron-dao";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//--------------------------------------------------------------------------------------------------
// Instantiation
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    OWNER.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;

    if msg.denom.is_empty() {
        return Err(ContractError::DenomTooShort);
    }
    DENOM.save(deps.storage, &msg.denom)?;

    Ok(Response::new())
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
) -> ContractResult<Response> {
    let api = deps.api;
    match msg {
        ExecuteMsg::TransferOwnership { new_owner } => {
            transfer_ownership(deps, info.sender, api.addr_validate(&new_owner)?)
        }
        ExecuteMsg::LockFunds {} => execute_lock_funds(deps, env, info),
        ExecuteMsg::UnlockFunds { amount } => execute_unlock_funds(deps, env, info, amount),
    }
}

fn transfer_ownership(
    deps: DepsMut,
    sender_addr: Addr,
    new_owner_addr: Addr,
) -> ContractResult<Response> {
    let owner_addr = OWNER.load(deps.storage)?;
    if sender_addr != owner_addr {
        return Err(ContractError::InsufficientPrivileges(
            "only owner can transfer ownership".into(),
        ));
    }

    OWNER.save(deps.storage, &new_owner_addr)?;

    Ok(Response::new()
        .add_attribute("action", "neutron/voting/transfer_ownership")
        .add_attribute("previous_owner", owner_addr)
        .add_attribute("new_owner", new_owner_addr))
}

fn execute_lock_funds(deps: DepsMut, _env: Env, info: MessageInfo) -> ContractResult<Response> {
    let denom = DENOM.load(deps.storage)?;
    let incoming_funds = info
        .funds
        .into_iter()
        .find(|fund| fund.denom == denom)
        .unwrap_or_else(|| coin(0, &denom));
    TOKENS_LOCKED.update::<_, ContractError>(deps.storage, &info.sender, |amount| {
        Ok(amount.unwrap_or_default() + incoming_funds.amount)
    })?;
    Ok(Response::new()
        .add_attribute("action", "lock_funds")
        .add_attribute("user", info.sender)
        .add_attribute("amount", format!("{}{}", incoming_funds.amount, denom)))
}

fn execute_unlock_funds(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    let locked = TOKENS_LOCKED.load(deps.storage, &info.sender)?;
    match amount.cmp(&locked) {
        Ordering::Less => {
            TOKENS_LOCKED.save(deps.storage, &info.sender, &(locked - amount))?;
        }
        Ordering::Equal => {
            TOKENS_LOCKED.remove(deps.storage, &info.sender);
        }
        Ordering::Greater => {
            return Err(ContractError::NotEnoughFundsToUnlock {
                requested: amount,
                has: locked,
            })
        }
    };
    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(amount.u128(), DENOM.load(deps.storage)?)],
        }))
        .add_attribute("action", "unlock_funds")
        .add_attribute("user", info.sender)
        .add_attribute("amount", format!("{}{}", amount, DENOM.load(deps.storage)?)))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let api = deps.api;
    Ok(match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?)?,
        QueryMsg::VotingPower { user } => {
            to_binary(&query_voting_power(deps, api.addr_validate(&user)?)?)?
        }
        QueryMsg::VotingPowers {} => to_binary(&query_voting_powers(deps)?)?,
    })
}

fn query_config(deps: Deps) -> ContractResult<ConfigResponse> {
    Ok(ConfigResponse {
        owner: OWNER.load(deps.storage)?.into(),
        denom: DENOM.load(deps.storage)?,
    })
}

fn query_voting_power(deps: Deps, user_addr: Addr) -> ContractResult<VotingPowerResponse> {
    let voting_power = match TOKENS_LOCKED.may_load(deps.storage, &user_addr)? {
        Some(voting_power) => voting_power,
        None => Uint128::zero(),
    };

    Ok(VotingPowerResponse {
        user: user_addr.to_string(),
        voting_power,
    })
}

fn query_voting_powers(deps: Deps) -> ContractResult<Vec<VotingPowerResponse>> {
    let voting_powers = TOKENS_LOCKED
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|res| {
            let (addr, voting_power) = res?;
            Ok(VotingPowerResponse {
                user: addr.to_string(),
                voting_power,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(voting_powers)
}
