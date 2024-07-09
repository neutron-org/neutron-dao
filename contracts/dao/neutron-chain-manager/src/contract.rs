use crate::cron_module_param_types::{
    MsgUpdateParamsCron, ParamsRequestCron, ParamsResponseCron, MSG_TYPE_UPDATE_PARAMS_CRON,
    PARAMS_QUERY_PATH_CRON,
};
use crate::dex_module_param_types::{
    MsgUpdateParamsDex, ParamsRequestDex, ParamsResponseDex, MSG_TYPE_UPDATE_PARAMS_DEX,
    PARAMS_QUERY_PATH_DEX
};
use crate::tokenfactory_module_param_types::{
    MsgUpdateParamsTokenfactory, ParamsRequestTokenfactory, ParamsResponseTokenfactory,
    MSG_TYPE_UPDATE_PARAMS_TOKENFACTORY, PARAMS_QUERY_PATH_TOKENFACTORY,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::{AdminProposal, NeutronMsg, ProposalExecuteMessage};
use neutron_sdk::proto_types::neutron::cron::QueryParamsRequest as QueryParamsRequestCron;
use neutron_sdk::proto_types::neutron::dex::QueryParamsRequest as QueryParamsRequestDex;
use neutron_sdk::proto_types::osmosis::tokenfactory::v1beta1::QueryParamsRequest as QueryParamsRequestTokenfactory;
use neutron_sdk::stargate::aux::make_stargate_query;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, Permission, ProposalExecuteMessageJSON, QueryMsg,
    Strategy, StrategyMsg,
};
use crate::state::STRATEGIES;

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

    STRATEGIES.save(
        deps.storage,
        msg.initial_strategy_address.clone(),
        &Strategy::AllowAll,
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("init_allow_all_address", msg.initial_strategy_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::AddStrategy { address, strategy } => {
            execute_add_strategy(deps, info, address, strategy)
        }
        ExecuteMsg::RemoveStrategy { address } => execute_remove_strategy(deps, info, address),
        ExecuteMsg::ExecuteMessages { messages } => execute_execute_messages(deps, info, messages),
    }
}

pub fn execute_add_strategy(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
    strategy: StrategyMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    is_authorized(deps.as_ref(), info.sender.clone())?;

    // We add the new strategy, and then we check that it did not replace
    // the only existing ALLOW_ALL strategy.
    STRATEGIES.save(deps.storage, address.clone(), &strategy.clone().into())?;
    if let StrategyMsg::AllowOnly(_) = strategy {
        if no_admins_left(deps.as_ref())? {
            return Err(ContractError::InvalidDemotion {});
        }
    }

    Ok(Response::new()
        .add_attribute("action", "execute_add_strategy")
        .add_attribute("address", address))
}

pub fn execute_remove_strategy(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
) -> Result<Response<NeutronMsg>, ContractError> {
    is_authorized(deps.as_ref(), info.sender.clone())?;

    // First we remove the strategy, then we check that it was not the only
    // ALLOW_ALL strategy we had.
    STRATEGIES.remove(deps.storage, address.clone());
    if no_admins_left(deps.as_ref())? {
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
    // If the sender doesn't have a strategy associated with them, abort immediately.
    if !STRATEGIES.has(deps.storage, info.sender.clone()) {
        return Err(ContractError::Unauthorized {});
    }

    let response = Response::new()
        .add_attribute("action", "execute_execute_messages")
        .add_attribute("address", info.sender.clone());

    let strategy = STRATEGIES.load(deps.storage, info.sender)?;
    match strategy {
        Strategy::AllowAll => Ok(response
            .add_attribute("strategy", "allow_all")
            .add_messages(messages)),
        Strategy::AllowOnly(_) => {
            check_allow_only_permissions(deps.as_ref(), strategy.clone(), messages.clone())?;
            Ok(response
                .add_attribute("strategy", "allow_only")
                .add_messages(messages))
        }
    }
}

fn is_authorized(deps: Deps, address: Addr) -> Result<(), ContractError> {
    match STRATEGIES.load(deps.storage, address) {
        Ok(Strategy::AllowAll) => Ok(()),
        _ => Err(ContractError::Unauthorized {}),
    }
}

/// This function returns true if there is no more allow_all strategies left.
fn no_admins_left(deps: Deps) -> Result<bool, ContractError> {
    let not_found: bool = !STRATEGIES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<Result<Vec<(Addr, Strategy)>, _>>()?
        .into_iter()
        .any(|(_, strategy)| matches!(strategy, Strategy::AllowAll));

    Ok(not_found)
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
        NeutronMsg::AddSchedule { .. } => {
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

    if typed_proposal.type_field.as_str() == MSG_TYPE_UPDATE_PARAMS_CRON {
        check_cron_update_msg_params(deps, strategy, proposal)?;
        Ok(())
    } else if typed_proposal.type_field.as_str() == MSG_TYPE_UPDATE_PARAMS_TOKENFACTORY {
        check_tokenfactory_update_msg_params(deps, strategy, proposal)?;
        Ok(())
    } else if typed_proposal.type_field.as_str() == MSG_TYPE_UPDATE_PARAMS_DEX {
        check_dex_update_msg_params(deps, strategy, proposal)?;
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
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
    make_stargate_query(
        deps,
        PARAMS_QUERY_PATH_CRON,
        QueryParamsRequestCron::from(req),
    )
}

/// Checks that the strategy owner is authorised to change the parameters of the
/// tokenfactory module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
fn check_tokenfactory_update_msg_params(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let msg_update_params: MsgUpdateParamsTokenfactory =
        serde_json_wasm::from_str(proposal.message.as_str())?;

    let tokenfactory_update_param_permission = strategy
        .get_tokenfactory_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let tokenfactory_params = get_tokenfactory_params(deps, ParamsRequestTokenfactory {})?;
    if tokenfactory_params.params.denom_creation_fee != msg_update_params.params.denom_creation_fee
        && !tokenfactory_update_param_permission.denom_creation_fee
    {
        return Err(ContractError::Unauthorized {});
    }

    if tokenfactory_params.params.denom_creation_gas_consume
        != msg_update_params.params.denom_creation_gas_consume
        && !tokenfactory_update_param_permission.denom_creation_gas_consume
    {
        return Err(ContractError::Unauthorized {});
    }

    if tokenfactory_params.params.fee_collector_address
        != msg_update_params.params.fee_collector_address
        && !tokenfactory_update_param_permission.fee_collector_address
    {
        return Err(ContractError::Unauthorized {});
    }

    if tokenfactory_params.params.whitelisted_hooks != msg_update_params.params.whitelisted_hooks
        && !tokenfactory_update_param_permission.whitelisted_hooks
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the tokenfactory module.
pub fn get_tokenfactory_params(
    deps: Deps,
    req: ParamsRequestTokenfactory,
) -> StdResult<ParamsResponseTokenfactory> {
    make_stargate_query(
        deps,
        PARAMS_QUERY_PATH_TOKENFACTORY,
        QueryParamsRequestTokenfactory::from(req),
    )
}

    /// Checks that the strategy owner is authorised to change the parameters of the
/// dex module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
fn check_dex_update_msg_params(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let msg_update_params: MsgUpdateParamsDex =
        serde_json_wasm::from_str(proposal.message.as_str())?;

    let dex_update_param_permission = strategy
        .get_dex_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let dex_params = get_dex_params(deps, ParamsRequestDex {})?;
    // TODO: do this through iteration instead of ifs for future proofing new params
    if dex_params.params.fee_tiers != msg_update_params.params.fee_tiers
        && !dex_update_param_permission.fee_tiers
    {
        return Err(ContractError::Unauthorized {});
    }
    if dex_params.params.paused != msg_update_params.params.paused
        && !dex_update_param_permission.paused
    {
        return Err(ContractError::Unauthorized {});
    }
    if dex_params.params.max_jits_per_block != msg_update_params.params.max_jits_per_block
        && !dex_update_param_permission.max_jits_per_block
    {
        return Err(ContractError::Unauthorized {});
    }
    if dex_params.params.good_til_purge_allowance
        != msg_update_params.params.good_til_purge_allowance
        && !dex_update_param_permission.good_til_purge_allowance
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the dex module.
pub fn get_dex_params(deps: Deps, req: ParamsRequestDex) -> StdResult<ParamsResponseDex> {
    make_stargate_query(
        deps,
        PARAMS_QUERY_PATH_DEX,
        QueryParamsRequestDex::from(req),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Strategies {} => to_json_binary(&query_strategies(deps)?),
    }
}

/// No pagination is added because it's unlikely that there is going
/// to be more than 10 strategies.
pub fn query_strategies(deps: Deps) -> StdResult<Vec<(Addr, StrategyMsg)>> {
    let all_strategies: Vec<(Addr, StrategyMsg)> = STRATEGIES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|v| match v {
            Ok((addr, Strategy::AllowAll)) => Ok((addr, StrategyMsg::AllowAll)),
            Ok((addr, Strategy::AllowOnly(permissions))) => Ok((
                addr,
                StrategyMsg::AllowOnly(permissions.values().cloned().collect::<Vec<Permission>>()),
            )),
            Err(e) => Err(e),
        })
        .collect::<Result<Vec<(Addr, StrategyMsg)>, _>>()?;
    Ok(all_strategies)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
