use crate::cron_module_param_types::{
    MsgUpdateParamsCron, ParamsRequestCron, ParamsResponseCron, PARAMS_QUERY_PATH_CRON,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::{AdminProposal, NeutronMsg, ProposalExecuteMessage};
use neutron_sdk::proto_types::neutron::cron::QueryParamsRequest;
use neutron_sdk::stargate::aux::make_stargate_query;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, ProposalExecuteMessageJSON, QueryMsg, Strategy,
};
use crate::state::{STRATEGIES_ALLOW_ALL, STRATEGIES_ALLOW_ONLY};

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-chain-manager";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if !msg.initial_strategy.permissions.is_empty() {
        return Err(ContractError::InvalidInitialStrategy {});
    }

    STRATEGIES_ALLOW_ALL.save(
        deps.storage,
        msg.initial_strategy.address.clone(),
        &msg.initial_strategy,
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("init_allow_all_address", msg.initial_strategy.address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::AddStrategy { strategy } => execute_add_strategy(deps, info, strategy),
        ExecuteMsg::RemoveStrategy { address } => execute_remove_strategy(deps, info, address),
        ExecuteMsg::ExecuteMessages { messages } => execute_execute_messages(deps, info, messages),
    }
}

pub fn execute_add_strategy(
    deps: DepsMut,
    info: MessageInfo,
    strategy: Strategy,
) -> Result<Response<NeutronMsg>, ContractError> {
    if !STRATEGIES_ALLOW_ALL.has(deps.storage, info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // An address cannot have both an ALLOW_ALL strategy and an ALLOW_ONLY
    // strategy associated with it.
    if strategy.permissions.is_empty() {
        STRATEGIES_ALLOW_ALL.save(deps.storage, strategy.address.clone(), &strategy)?;
        // If an address was *promoted* to an ALLOW_ALL permission strategy,
        // we remove its ALLOW_ONLY entry.
        STRATEGIES_ALLOW_ONLY.remove(deps.storage, strategy.address.clone());
    } else {
        STRATEGIES_ALLOW_ONLY.save(deps.storage, strategy.address.clone(), &strategy)?;

        // If an address was *demoted* to an ALLOW_ONLY permission strategy, we
        // remove its ALLOW_ALL entry. If this operation leaves us without a
        // single ALLOW_ALL strategy, we abort the operation.
        STRATEGIES_ALLOW_ALL.remove(deps.storage, strategy.address.clone());
        if STRATEGIES_ALLOW_ALL.is_empty(deps.storage) {
            return Err(ContractError::InvalidDemotion {});
        }
    }

    Ok(Response::new()
        .add_attribute("action", "execute_add_strategy")
        .add_attribute("address", strategy.address)
        .add_attribute("permissions_count", strategy.permissions.len().to_string()))
}

pub fn execute_remove_strategy(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
) -> Result<Response<NeutronMsg>, ContractError> {
    if !STRATEGIES_ALLOW_ALL.has(deps.storage, info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    STRATEGIES_ALLOW_ONLY.remove(deps.storage, address.clone());
    // If this operation leaves us without a single ALLOW_ALL strategy, we
    // abort the operation.
    STRATEGIES_ALLOW_ALL.remove(deps.storage, address.clone());
    if STRATEGIES_ALLOW_ALL.is_empty(deps.storage) {
        return Err(ContractError::InvalidDemotion {});
    }

    Ok(Response::new()
        .add_attribute("action", "execute_remove_strategy")
        .add_attribute("address", address))
}

pub fn execute_execute_messages(
    deps: DepsMut,
    info: MessageInfo,
    messages: Vec<CosmosMsg<NeutronMsg>>,
) -> Result<Response<NeutronMsg>, ContractError> {
    if STRATEGIES_ALLOW_ALL.has(deps.storage, info.sender.clone()) {
        return Ok(Response::new()
            .add_attribute("action", "execute_execute_messages")
            .add_attribute("strategy", "allow_all")
            .add_attribute("address", info.sender.clone())
            .add_messages(messages));
    }

    // If the sender doesn't have a strategy associated with them, abort immediately.
    if !STRATEGIES_ALLOW_ONLY.has(deps.storage, info.sender.clone()) {
        return Err(ContractError::Unauthorized {});
    }

    let strategy = STRATEGIES_ALLOW_ONLY.load(deps.storage, info.sender.clone())?;
    check_allow_only_permissions(deps.as_ref(), strategy, messages.clone())?;

    Ok(Response::new()
        .add_attribute("action", "execute_execute_messages")
        .add_attribute("strategy", "allow_only")
        .add_attribute("address", info.sender.clone())
        .add_messages(messages))
}

/// For every message, check whether we have the permission to execute it.
/// Any missing permission aborts the execution. Trying to execute any
/// unknown message aborts the execution.
fn check_allow_only_permissions(
    deps: Deps,
    strategy: Strategy,
    messages: Vec<CosmosMsg<NeutronMsg>>,
) -> Result<(), ContractError> {
    for msg in messages.clone() {
        if let CosmosMsg::Custom(neutron_msg) = msg {
            check_neutron_msg(deps, strategy.clone(), neutron_msg)?
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }

    Ok(())
}

fn check_neutron_msg(
    deps: Deps,
    strategy: Strategy,
    neutron_msg: NeutronMsg,
) -> Result<(), ContractError> {
    match neutron_msg {
        NeutronMsg::AddSchedule {
            name: _,
            period: _,
            msgs: _,
        } => {
            if !strategy.has_cron_add_schedule_permission() {
                return Err(ContractError::Unauthorized {});
            }
        }
        NeutronMsg::RemoveSchedule { name: _ } => {
            if !strategy.has_cron_remove_schedule_permission() {
                return Err(ContractError::Unauthorized {});
            }
        }
        NeutronMsg::SubmitAdminProposal { admin_proposal } => {
            check_submit_admin_proposal_message(deps, strategy, admin_proposal)?;
        }
        _ => {
            return Err(ContractError::Unauthorized {});
        }
    }

    Ok(())
}

fn check_submit_admin_proposal_message(
    deps: Deps,
    strategy: Strategy,
    proposal: AdminProposal,
) -> Result<(), ContractError> {
    match proposal {
        AdminProposal::ParamChangeProposal(proposal) => {
            for param_change in proposal.param_changes {
                if !strategy.has_param_change_permission(param_change) {
                    return Err(ContractError::Unauthorized {});
                }
            }
        }
        AdminProposal::ProposalExecuteMessage(proposal) => {
            check_proposal_execute_message(deps, strategy.clone(), proposal)?;
        }
        _ => {
            return Err(ContractError::Unauthorized {});
        }
    }

    Ok(())
}

/// Processes ProposalExecuteMessage messages. Message type has to be checked
/// as a string; after that, you can parse the JSON payload into a specific
/// message.
fn check_proposal_execute_message(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let typed_proposal: ProposalExecuteMessageJSON =
        serde_json_wasm::from_str(proposal.message.as_str())?;

    if typed_proposal.type_field.as_str() == "/neutron.cron.MsgUpdateParams" {
        check_cron_update_msg_params(deps, strategy, proposal)?;
    }

    Ok(())
}
/// Checks that the strategy owner is authorised to change the parameters of the
/// cron module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
fn check_cron_update_msg_params(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let msg_update_params: MsgUpdateParamsCron =
        serde_json_wasm::from_str(proposal.message.as_str())?;

    let cron_update_param_permission = strategy
        .get_cron_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let cron_params = get_cron_params(deps, ParamsRequestCron {})?;
    if cron_params.params.limit != msg_update_params.params.limit
        && !cron_update_param_permission.limit
    {
        return Err(ContractError::Unauthorized {});
    }

    if cron_params.params.security_address != msg_update_params.params.security_address
        && !cron_update_param_permission.security_address
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the cron module.
pub fn get_cron_params(deps: Deps, req: ParamsRequestCron) -> StdResult<ParamsResponseCron> {
    make_stargate_query(deps, PARAMS_QUERY_PATH_CRON, QueryParamsRequest::from(req))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Strategies {} => to_json_binary(&query_strategies(deps)?),
    }
}

/// No pagination is added because it's unlikely that there is going
/// to be more than 10 strategies.
pub fn query_strategies(deps: Deps) -> StdResult<Vec<Strategy>> {
    let mut all_strategies: Vec<Strategy> = vec![];

    let allow_all_strategies = STRATEGIES_ALLOW_ALL
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect::<StdResult<Vec<Strategy>>>()?;

    let allow_only_strategies = STRATEGIES_ALLOW_ONLY
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect::<StdResult<Vec<Strategy>>>()?;

    all_strategies.extend(allow_all_strategies);
    all_strategies.extend(allow_only_strategies);

    Ok(all_strategies)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
