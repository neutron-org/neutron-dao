#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cwd_pre_propose_base::msg::QueryMsg as PreProposeQueryBase;
use neutron_bindings::bindings::msg::NeutronMsg;
use neutron_subdao_core::msg::QueryMsg as SubdaoQuery;
use neutron_subdao_pre_propose_single::msg::QueryMsg as PreProposeQuery;
use neutron_subdao_timelock_single::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use neutron_subdao_timelock_single::types::{
    Config, ProposalListResponse, ProposalStatus, SingleChoiceProposal,
};

use crate::error::ContractError;
use crate::state::{CONFIG, DEFAULT_LIMIT, PROPOSALS};

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
        &PreProposeQuery::QueryBase(PreProposeQueryBase::Dao {}),
    )?;

    let main_dao: Addr = deps
        .querier
        .query_wasm_smart(subdao_core.clone(), &SubdaoQuery::MainDao {})?;

    let config = Config {
        owner: main_dao,
        timelock_duration: msg.timelock_duration,
        subdao: subdao_core,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", config.owner)
        .add_attribute("timelock_duration", config.timelock_duration.to_string()))
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
            timelock_duration,
        } => execute_update_config(deps, info, owner, timelock_duration),
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
        timelock_ts: env.block.time,
        status: ProposalStatus::Timelocked,
    };

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    // todo!(oldremez) send overrule creation proposal message

    Ok(Response::default()
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

    // Check if timelock has passed
    if env.block.time.seconds() < (config.timelock_duration + proposal.timelock_ts.seconds()) {
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
    new_timelock_duration: Option<u64>,
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

    if let Some(timelock_duration) = new_timelock_duration {
        config.timelock_duration = timelock_duration;
    }

    // TODO(oopcode): implement updating the .sudbao parameter.

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner)
        .add_attribute("timelock_duration", config.timelock_duration.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Proposal { proposal_id } => query_proposal(deps, proposal_id),
        QueryMsg::ListProposals { start_after, limit } => {
            query_list_proposals(deps, start_after, limit)
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
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

    Ok(Response::new().add_attribute(
        "timelocked_proposal_execution_failed",
        proposal_id.to_string(),
    ))
}
