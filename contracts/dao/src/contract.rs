#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::OWNER;
use neutron_bindings::errors::{NeutronError, NeutronResult};
use neutron_bindings::msg::{NeutronMsg, ParamChange, ParamChangeProposal, TextProposal};

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
) -> NeutronResult<Response<NeutronMsg>> {
    let api = deps.api;
    match msg {
        ExecuteMsg::TransferOwnership { new_owner } => {
            transfer_ownership(deps, info.sender, api.addr_validate(&new_owner)?)
        }
        ExecuteMsg::AddAdmin { new_admin } => execute_add_admin(new_admin),
        ExecuteMsg::SubmitTextProposal { title, description } => {
            execute_submit_text_proposal(title, description)
        }
        ExecuteMsg::SubmitChangeParamsProposal {
            title,
            description,
            params_change,
        } => execute_submit_param_change_proposal(title, description, params_change),
    }
}

pub fn transfer_ownership(
    deps: DepsMut,
    sender_addr: Addr,
    new_owner_addr: Addr,
) -> NeutronResult<Response<NeutronMsg>> {
    let owner_addr = OWNER.load(deps.storage)?;
    if sender_addr != owner_addr {
        return Err(NeutronError::Std(StdError::generic_err(
            "only owner can transfer ownership",
        )));
    }

    OWNER.save(deps.storage, &new_owner_addr)?;

    Ok(Response::new()
        .add_attribute("action", "neutron/dao/transfer_ownership")
        .add_attribute("previous_owner", owner_addr)
        .add_attribute("new_owner", new_owner_addr))
}

pub fn execute_add_admin(admin: String) -> NeutronResult<Response<NeutronMsg>> {
    let msg = NeutronMsg::add_admin(admin);
    Ok(Response::new().add_message(msg))
}

pub fn execute_submit_text_proposal(
    title: String,
    description: String,
) -> NeutronResult<Response<NeutronMsg>> {
    let proposal = TextProposal { title, description };

    let msg = NeutronMsg::submit_text_proposal(proposal);
    Ok(Response::new().add_message(msg))
}

pub fn execute_submit_param_change_proposal(
    title: String,
    description: String,
    param_changes: Vec<ParamChange>,
) -> NeutronResult<Response<NeutronMsg>> {
    let proposal = ParamChangeProposal {
        title,
        description,
        param_changes,
    };

    let msg = NeutronMsg::submit_param_change_proposal(proposal);
    Ok(Response::new().add_message(msg))
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
