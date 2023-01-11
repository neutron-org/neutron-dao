#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use exec_control::pause::{
    can_pause, can_unpause, validate_duration, PauseError, PauseInfoResponse,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, PAUSED_UNTIL};

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
        main_dao_address: deps.api.addr_validate(&msg.main_dao_address)?,
        security_dao_address: deps.api.addr_validate(&msg.security_dao_address)?,
    };
    CONFIG.save(deps.storage, &config)?;
    PAUSED_UNTIL.save(deps.storage, &None)?;

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
) -> Result<Response, ContractError> {
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

    let api = deps.api;
    match msg {
        ExecuteMsg::Pause { duration } => execute_pause(deps, env, info.sender, duration),
        ExecuteMsg::Unpause {} => execute_unpause(deps, info.sender),
        // permissioned - owner
        ExecuteMsg::TransferOwnership(new_owner) => {
            execute_transfer_ownership(deps, info, api.addr_validate(&new_owner)?)
        }
        ExecuteMsg::Payout { amount, recipient } => {
            execute_payout(deps, info, env, amount, recipient)
        }
    }
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

pub fn execute_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_main_dao_address: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sender_addr = info.sender;
    let old_main_dao_address = config.main_dao_address;
    if sender_addr != old_main_dao_address {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.main_dao_address = new_main_dao_address.clone();
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "neutron/treasury/transfer_ownership")
        .add_attribute("previous_owner", old_main_dao_address)
        .add_attribute("new_owner", new_main_dao_address))
}

pub fn execute_payout(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    amount: Uint128,
    recipient: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    if info.sender != config.main_dao_address {
        return Err(ContractError::Unauthorized {});
    }
    // verify that the contract has enough funds
    let bank_balance = deps
        .querier
        .query_balance(env.contract.address, &denom)?
        .amount;

    if amount.gt(&bank_balance) {
        return Err(ContractError::InsufficientFunds {});
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::PauseInfo {} => query_paused(deps, env),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}
pub fn query_paused(deps: Deps, env: Env) -> StdResult<Binary> {
    to_binary(&get_pause_info(deps, &env)?)
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
