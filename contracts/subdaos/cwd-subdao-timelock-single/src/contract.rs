#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cwd_proposal_single::{
    msg::QueryMsg as MainDaoProposalModuleQueryMsg,
    query::ProposalResponse as MainDaoProposalResponse,
};
use cwd_voting::status::Status;
use neutron_dao_pre_propose_overrule::msg::{
    ExecuteMsg as OverruleExecuteMsg, ProposeMessage as OverruleProposeMessage,
    QueryExt as OverruleQueryExt, QueryMsg as OverruleQueryMsg,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_subdao_core::msg::QueryMsg as SubdaoQuery;
use neutron_subdao_pre_propose_single::msg::QueryMsg as PreProposeQuery;
use neutron_subdao_timelock_single::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    types::{Config, ProposalListResponse, ProposalStatus, SingleChoiceProposal},
};

use crate::error::ContractError;
use crate::state::{CONFIG, DEFAULT_LIMIT, PROPOSALS, PROPOSAL_FAILED_EXECUTION_ERRORS};

pub(crate) const CONTRACT_NAME: &str = "crates.io:cwd-subdao-timelock-single";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let subdao_core: Addr = deps.querier.query_wasm_smart(
        info.sender, // sender is meant to be the pre-propose module
        &PreProposeQuery::Dao {},
    )?;

    let main_dao: Addr = deps
        .querier
        .query_wasm_smart(subdao_core.clone(), &SubdaoQuery::MainDao {})?;

    // We don't validate overrule pre propose address more than just as address.
    // We could also query the DAO address of this module and check if it matches the main DAO set
    // in the config. But we don't do that because it would require for the subdao to know much more
    // about the main DAO than it should IMO. It also makes testing harder.
    let overrule_pre_propose = deps.api.addr_validate(&msg.overrule_pre_propose)?;

    let config = Config {
        owner: main_dao,
        overrule_pre_propose,
        subdao: subdao_core,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", config.owner)
        .add_attribute(
            "overrule_pre_propose",
            config.overrule_pre_propose.to_string(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::TimelockProposal { proposal_id, msgs } => {
            execute_timelock_proposal(deps, env, info, proposal_id, msgs)
        }
        ExecuteMsg::ExecuteProposal { proposal_id } => {
            execute_execute_proposal(deps, env, info, proposal_id)
        }
        ExecuteMsg::OverruleProposal { proposal_id } => {
            execute_overrule_proposal(deps, info, proposal_id)
        }
        ExecuteMsg::UpdateConfig {
            owner,
            overrule_pre_propose,
        } => execute_update_config(deps, info, owner, overrule_pre_propose),
    }
}

pub fn execute_timelock_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    msgs: Vec<CosmosMsg<NeutronMsg>>,
) -> Result<Response<NeutronMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.subdao != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let proposal = SingleChoiceProposal {
        id: proposal_id,
        msgs,
        status: ProposalStatus::Timelocked,
    };

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    let create_overrule_proposal = WasmMsg::Execute {
        contract_addr: config.overrule_pre_propose.to_string(),
        msg: to_binary(&OverruleExecuteMsg::Propose {
            msg: OverruleProposeMessage::ProposeOverrule {
                timelock_contract: env.contract.address.to_string(),
                proposal_id,
            },
        })?,
        funds: vec![],
    };

    // NOTE: we don't handle an error that might appear during overrule proposal creation.
    // Thus, we expect for proposal to get execution error status on proposal module level.
    Ok(Response::default()
        .add_message(create_overrule_proposal)
        .add_attribute("action", "timelock_proposal")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", proposal.status.to_string()))
}

pub fn execute_execute_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response<NeutronMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    // Check if proposal is timelocked
    if proposal.status != ProposalStatus::Timelocked {
        return Err(ContractError::WrongStatus {
            status: proposal.status.to_string(),
        });
    }

    if !is_overrule_proposal_rejected(&deps, &env, &config.overrule_pre_propose, proposal.id)? {
        return Err(ContractError::TimeLocked {});
    }

    // Update proposal status
    proposal.status = ProposalStatus::Executed;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    let msgs: Vec<SubMsg<NeutronMsg>> = proposal
        .msgs
        .iter()
        .map(|msg| SubMsg::reply_on_error(msg.clone(), proposal_id))
        .collect();

    // Note: we add the proposal messages as submessages to change the status to ExecutionFailed
    // in the reply handler if any of the submessages fail.
    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("action", "execute_proposal")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string()))
}

pub fn execute_overrule_proposal(
    deps: DepsMut,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response<NeutronMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check if sender is owner; the owner is supposed to be the main Neutron DAO.
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    // Check if proposal is timelocked
    if proposal.status != ProposalStatus::Timelocked {
        return Err(ContractError::WrongStatus {
            status: proposal.status.to_string(),
        });
    }

    // Update proposal status
    proposal.status = ProposalStatus::Overruled;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::default()
        .add_attribute("action", "overrule_proposal")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string()))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Option<String>,
    new_overrule_pre_propose: Option<String>,
) -> Result<Response<NeutronMsg>, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = new_owner
        .map(|new_owner| deps.api.addr_validate(&new_owner))
        .transpose()?;

    if let Some(owner) = new_owner {
        config.owner = owner;
    }

    if let Some(overrule_pre_propose) = new_overrule_pre_propose {
        config.overrule_pre_propose = deps.api.addr_validate(&overrule_pre_propose)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner)
        .add_attribute(
            "overrule_pre_propose",
            config.overrule_pre_propose.to_string(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Proposal { proposal_id } => query_proposal(deps, proposal_id),
        QueryMsg::ListProposals { start_after, limit } => {
            query_list_proposals(deps, start_after, limit)
        }
        QueryMsg::ProposalFailedExecutionError { proposal_id } => {
            query_proposal_failed_execution_error(deps, proposal_id)
        }
    }
}

pub fn query_proposal(deps: Deps, id: u64) -> StdResult<Binary> {
    let proposal = PROPOSALS.load(deps.storage, id)?;
    to_binary(&proposal)
}

pub fn query_list_proposals(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Binary> {
    let min = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let props: Vec<SingleChoiceProposal> = PROPOSALS
        .range(deps.storage, min, None, cosmwasm_std::Order::Ascending)
        .take(limit as usize)
        .collect::<Result<Vec<(u64, SingleChoiceProposal)>, _>>()?
        .into_iter()
        .map(|(_, proposal)| proposal)
        .collect();

    to_binary(&ProposalListResponse { proposals: props })
}

fn query_proposal_failed_execution_error(deps: Deps, proposal_id: u64) -> StdResult<Binary> {
    let proposal_error = PROPOSAL_FAILED_EXECUTION_ERRORS.may_load(deps.storage, proposal_id)?;
    to_binary(&proposal_error)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

fn is_overrule_proposal_rejected(
    deps: &DepsMut,
    env: &Env,
    overrule_pre_propose: &Addr,
    subdao_proposal_id: u64,
) -> Result<bool, ContractError> {
    let overrule_proposal_id: u64 = deps.querier.query_wasm_smart(
        overrule_pre_propose,
        &OverruleQueryMsg::QueryExtension {
            msg: OverruleQueryExt::OverruleProposalId {
                timelock_address: env.contract.address.to_string(),
                subdao_proposal_id,
            },
        },
    )?;
    let propose: Addr = deps
        .querier
        .query_wasm_smart(overrule_pre_propose, &OverruleQueryMsg::ProposalModule {})?;
    let overrule_proposal: MainDaoProposalResponse = deps.querier.query_wasm_smart(
        propose,
        &MainDaoProposalModuleQueryMsg::Proposal {
            proposal_id: overrule_proposal_id,
        },
    )?;
    Ok(overrule_proposal.proposal.status == Status::Rejected)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let proposal_id = msg.id;

    PROPOSALS.update(deps.storage, proposal_id, |prop| match prop {
        Some(mut prop) => {
            prop.status = ProposalStatus::ExecutionFailed;
            Ok(prop)
        }
        None => Err(ContractError::NoSuchProposal { id: proposal_id }),
    })?;

    let err = msg
        .result
        .into_result()
        .err()
        .unwrap_or_else(|| "result is not error".to_string());
    // Error is reduced before cosmwasm reply and is expected in form of "codespace=? code=?"
    PROPOSAL_FAILED_EXECUTION_ERRORS.save(deps.storage, proposal_id, &err)?;

    Ok(Response::new().add_attribute(
        "timelocked_proposal_execution_failed",
        proposal_id.to_string(),
    ))
}
