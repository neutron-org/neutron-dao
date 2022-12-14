#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

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
        owner: deps.api.addr_validate(&msg.owner)?,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let api = deps.api;
    match msg {
        // permissioned - owner
        ExecuteMsg::TransferOwnership(new_owner) => {
            execute_transfer_ownership(deps, info, api.addr_validate(&new_owner)?)
        }
        ExecuteMsg::Payout { amount, recipient } => {
            execute_payout(deps, info, env, amount, recipient)
        }
    }
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner_addr: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let sender_addr = info.sender;
    let old_owner = config.owner;
    if sender_addr != old_owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner = new_owner_addr.clone();
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "neutron/treasury/transfer_ownership")
        .add_attribute("previous_owner", old_owner)
        .add_attribute("new_owner", new_owner_addr))
}

pub fn execute_payout(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    amount: Uint128,
    recipient: String,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }
    // verify that the contract has enough funds
    let bank_balance = deps
        .querier
        .query_balance(env.contract.address, denom.clone())?
        .amount;

    if amount.gt(&bank_balance) {
        return Err(StdError::generic_err("insufficient funds"));
    }

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.clone(),
            amount: coins(amount.u128(), denom),
        }))
        .add_attribute("action", "neutron/treasury/payout")
        .add_attribute("amount", amount)
        .add_attribute("recipient", recipient))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}
