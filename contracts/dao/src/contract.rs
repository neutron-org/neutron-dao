
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, MsgTextProposal};
use crate::state::{OWNER};
use neutron_bindings::msg::{NeutronMsg, TextProposal, ParamChangeProposal, ParamChange, SoftwareUpdateProposal, ClientUpdateSpendProposal, CommunitySpendProposal, CancelSoftwareUpdateProposal};
use neutron_bindings::ProtobufAny;

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
        ExecuteMsg::AddAdmin(admin) => execute_add_admin(admin),
        ExecuteMsg::SubmitTextProposal(title, description) => execute_submit_text_proposal(title, text),
        ExecuteMsg::SubmitChangeParamProposal(title, description, params_change) => execute_submit_param_change_proposal(title, description, params_change),
        ExecuteMsg::SubmitCommunityPoolSpendProposal(title, description, recipient) => execute_submit_community_pool_spend_proposal(title,description,recipient),
        ExecuteMsg::SubmitSoftwareUpdateProposal(title, description) => execute_software_update_proposal(title, description),
        ExecuteMsg::SubmitCancelSoftwareUpdateProposal(title, description) => execute_cancel_software_update_proposal(title, description),
        ExecuteMsg::SubmitClientUpdateProposal(title, description, subject_client_id, substitute_client_id) => execute_client_update_proposal(title, description, subject_client_id, substitute_client_id)

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
        .add_attribute("action", "neutron/dao/transfer_ownership")
        .add_attribute("previous_owner", owner_addr)
        .add_attribute("new_owner", new_owner_addr))
}

pub fn execute_add_admin(admin: String) -> StdResult<Response> {
    NeutronMsg::add_admin(admin);
    Ok(Response::default())
}


pub fn execute_submit_text_proposal(title: String, description: String) -> StdResult<Response> {
    let proposal = TextProposal{
        title,
        description,
    };

    NeutronMsg::submit_text_proposal(
        proposal,
    );
    Ok(Response::default())
}

pub fn execute_submit_param_change_proposal(title: String, description: String, param_changes: Vec<ParamChange>) -> StdResult<Response> {
    let proposal = ParamChangeProposal{
        title,
        description,
        param_changes,
    };

    NeutronMsg::submit_param_change_proposal(
        proposal,
    );
    Ok(Response::default())
}

pub fn execute_submit_community_pool_spend_proposal(title: String, description: String, recipient: String) -> StdResult<Response> {
    let proposal = CommunitySpendProposal{
        title,
        description,
        recipient
    };

    NeutronMsg::submit_community_spend_proposal(
        proposal,
    );
    Ok(Response::default())
}

pub fn execute_client_update_proposal(title: String, description: String, subject_client_id: String, substitute_client_id: String) -> StdResult<Response> {
    let proposal = ClientUpdateSpendProposal{
        title,
        description,
        subject_client_id,
        substitute_client_id
    };

    NeutronMsg::submiit_client_update_spend_proposal(
        proposal,
    );
    Ok(Response::default())
}
pub fn execute_software_update_proposal(title: String, description: String) -> StdResult<Response> {
    let proposal = SoftwareUpdateProposal{
        title,
        description,
    };

    NeutronMsg::submit_software_update_proposal(
        proposal,
    );
    Ok(Response::default())
}

pub fn execute_cancel_software_update_proposal(title: String, description: String) -> StdResult<Response> {
    let proposal = CancelSoftwareUpdateProposal{
        title,
        description,
    };

    NeutronMsg::submit_cancel_software_update_proposal(
        proposal,
    );
    Ok(Response::default())
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

