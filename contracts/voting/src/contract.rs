#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, VotingPowerResponse};
use crate::state::{OWNER};
use neutron_bindings::msg::NeutronMsg;

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
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    OWNER.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;

    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let api = deps.api;
    match msg {
        ExecuteMsg::TransferOwnership(new_owner) => {
            transfer_ownership(deps, info.sender, api.addr_validate(&new_owner)?)
        }
        ExecuteMsg::AddAdmin(admin) => add_admin(admin),
    }
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
        .add_attribute("action", "neutron/voting/transfer_ownership")
        .add_attribute("previous_owner", owner_addr)
        .add_attribute("new_owner", new_owner_addr))
}

pub fn add_admin(admin: admin) -> StdResult<Response> {
    NeutronMsg::add_admin(admin)
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

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        owner: OWNER.load(deps.storage)?.into(),
    })
}

