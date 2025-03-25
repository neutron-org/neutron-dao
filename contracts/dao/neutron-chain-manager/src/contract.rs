#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::{AdminProposal, NeutronMsg, ProposalExecuteMessage};
use neutron_std::types::cosmos::upgrade::v1beta1::{MsgCancelUpgrade, MsgSoftwareUpgrade};
use neutron_std::types::gaia::globalfee;
use neutron_std::types::interchain_security::ccv::consumer;
use neutron_std::types::neutron::cron;
use neutron_std::types::neutron::dex;
use neutron_std::types::neutron::dynamicfees;
use neutron_std::types::osmosis::tokenfactory;

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
    match typed_proposal.type_field.as_str() {
        cron::MsgUpdateParams::TYPE_URL => {
            check_cron_update_msg_params(deps, strategy, proposal)?;
            Ok(())
        }
        tokenfactory::v1beta1::MsgUpdateParams::TYPE_URL => {
            check_tokenfactory_update_msg_params(deps, strategy, proposal)?;
            Ok(())
        }
        dex::MsgUpdateParams::TYPE_URL => {
            check_dex_update_msg_params(deps, strategy, proposal)?;
            Ok(())
        }
        dynamicfees::v1::MsgUpdateParams::TYPE_URL => {
            check_dynamicfees_update_msg_params(deps, strategy, proposal)?;
            Ok(())
        }
        globalfee::v1beta1::MsgUpdateParams::TYPE_URL => {
            check_globalfee_update_msg_params(deps, strategy, proposal)?;
            Ok(())
        }
        consumer::v1::MsgUpdateParams::TYPE_URL => {
            check_ccv_update_msg_params(deps, strategy, proposal)?;
            Ok(())
        }
        cron::MsgAddSchedule::TYPE_URL => match strategy.has_cron_add_schedule_permission() {
            true => Ok(()),
            false => Err(ContractError::Unauthorized {}),
        },
        cron::MsgRemoveSchedule::TYPE_URL => match strategy.has_cron_remove_schedule_permission() {
            true => Ok(()),
            false => Err(ContractError::Unauthorized {}),
        },
        MsgSoftwareUpgrade::TYPE_URL => match strategy.has_software_upgrade_permission() {
            true => Ok(()),
            false => Err(ContractError::Unauthorized {}),
        },
        MsgCancelUpgrade::TYPE_URL => match strategy.has_cancel_software_upgrade_permission() {
            true => Ok(()),
            false => Err(ContractError::Unauthorized {}),
        },
        _ => Err(ContractError::Unauthorized {}),
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
    let msg_update_params =
        serde_json_wasm::from_str::<cron::MsgUpdateParams>(proposal.message.as_str())?
            .params
            .ok_or(ContractError::Unauthorized {})?;
    let cron_update_param_permission = strategy
        .get_cron_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let cron_params = get_cron_params(deps)?.params.unwrap_or_default();
    if cron_params.limit != msg_update_params.limit && !cron_update_param_permission.limit {
        return Err(ContractError::Unauthorized {});
    }

    if cron_params.security_address != msg_update_params.security_address
        && !cron_update_param_permission.security_address
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the cron module.
pub fn get_cron_params(deps: Deps) -> StdResult<cron::QueryParamsResponse> {
    let cron_querier = cron::CronQuerier::new(&deps.querier);
    cron_querier.params()
}

/// Checks that the strategy owner is authorised to change the parameters of the
/// tokenfactory module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
fn check_tokenfactory_update_msg_params(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let msg_update_params = serde_json_wasm::from_str::<tokenfactory::v1beta1::MsgUpdateParams>(
        proposal.message.as_str(),
    )?
    .params
    .ok_or(ContractError::Unauthorized {})?;
    let tokenfactory_update_param_permission = strategy
        .get_tokenfactory_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let tokenfactory_params = get_tokenfactory_params(deps)?.params.unwrap_or_default();
    if tokenfactory_params.denom_creation_fee != msg_update_params.denom_creation_fee
        && !tokenfactory_update_param_permission.denom_creation_fee
    {
        return Err(ContractError::Unauthorized {});
    }

    if tokenfactory_params.denom_creation_gas_consume
        != msg_update_params.denom_creation_gas_consume
        && !tokenfactory_update_param_permission.denom_creation_gas_consume
    {
        return Err(ContractError::Unauthorized {});
    }

    if tokenfactory_params.fee_collector_address != msg_update_params.fee_collector_address
        && !tokenfactory_update_param_permission.fee_collector_address
    {
        return Err(ContractError::Unauthorized {});
    }

    if tokenfactory_params.whitelisted_hooks != msg_update_params.whitelisted_hooks
        && !tokenfactory_update_param_permission.whitelisted_hooks
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the tokenfactory module.
pub fn get_tokenfactory_params(
    deps: Deps,
) -> StdResult<tokenfactory::v1beta1::QueryParamsResponse> {
    let factory_querier = tokenfactory::v1beta1::TokenfactoryQuerier::new(&deps.querier);
    factory_querier.params()
}

/// Checks that the strategy owner is authorised to change the parameters of the
/// dex module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
fn check_dex_update_msg_params(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let msg_update_params =
        serde_json_wasm::from_str::<dex::MsgUpdateParams>(proposal.message.as_str())?
            .params
            .ok_or(ContractError::Unauthorized {})?;

    let dex_update_param_permission = strategy
        .get_dex_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let dex_params = get_dex_params(deps)?.params.unwrap_or_default();

    if dex_params.fee_tiers != msg_update_params.fee_tiers && !dex_update_param_permission.fee_tiers
    {
        return Err(ContractError::Unauthorized {});
    }
    if dex_params.paused != msg_update_params.paused && !dex_update_param_permission.paused {
        return Err(ContractError::Unauthorized {});
    }
    if dex_params.max_jits_per_block != msg_update_params.max_jits_per_block
        && !dex_update_param_permission.max_jits_per_block
    {
        return Err(ContractError::Unauthorized {});
    }
    if dex_params.good_til_purge_allowance != msg_update_params.good_til_purge_allowance
        && !dex_update_param_permission.good_til_purge_allowance
    {
        return Err(ContractError::Unauthorized {});
    }

    if dex_params.whitelisted_lps != msg_update_params.whitelisted_lps
        && !dex_update_param_permission.whitelisted_lps
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the dex module.
pub fn get_dex_params(deps: Deps) -> StdResult<dex::QueryParamsResponse> {
    let dex_querier = dex::DexQuerier::new(&deps.querier);
    dex_querier.params()
}

/// Checks that the strategy owner is authorised to change the parameters of the
/// dynamicfees module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
fn check_dynamicfees_update_msg_params(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let msg_update_params =
        serde_json_wasm::from_str::<dynamicfees::v1::MsgUpdateParams>(proposal.message.as_str())?
            .params
            .ok_or(ContractError::Unauthorized {})?;

    let dynamicfees_update_param_permission = strategy
        .get_dynamicfees_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let dynamicfees_params = get_dynamicfees_params(deps)?.params.unwrap_or_default();

    if dynamicfees_params.ntrn_prices != msg_update_params.ntrn_prices
        && !dynamicfees_update_param_permission.ntrn_prices
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the dynamicfees module.
pub fn get_dynamicfees_params(deps: Deps) -> StdResult<dynamicfees::v1::QueryParamsResponse> {
    let dynamicfees_querier = dynamicfees::v1::DynamicfeesQuerier::new(&deps.querier);
    dynamicfees_querier.params()
}

/// Checks that the strategy owner is authorised to change the parameters of the
/// globalfee module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
fn check_globalfee_update_msg_params(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let msg_update_params = serde_json_wasm::from_str::<globalfee::v1beta1::MsgUpdateParams>(
        proposal.message.as_str(),
    )?
    .params
    .ok_or(ContractError::Unauthorized {})?;

    let globalfee_update_param_permission = strategy
        .get_globalfee_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let globalfee_params = get_globalfee_params(deps)?.params.unwrap_or_default();

    if globalfee_params.bypass_min_fee_msg_types != msg_update_params.bypass_min_fee_msg_types
        && !globalfee_update_param_permission.bypass_min_fee_msg_types
    {
        return Err(ContractError::Unauthorized {});
    }

    if globalfee_params.max_total_bypass_min_fee_msg_gas_usage
        != msg_update_params.max_total_bypass_min_fee_msg_gas_usage
        && !globalfee_update_param_permission.max_total_bypass_min_fee_msg_gas_usage
    {
        return Err(ContractError::Unauthorized {});
    }

    if globalfee_params.minimum_gas_prices != msg_update_params.minimum_gas_prices
        && !globalfee_update_param_permission.minimum_gas_prices
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the globalfee module.
pub fn get_globalfee_params(deps: Deps) -> StdResult<globalfee::v1beta1::QueryParamsResponse> {
    let globalfee_querier = globalfee::v1beta1::GlobalfeeQuerier::new(&deps.querier);
    globalfee_querier.params()
}

/// Checks that the strategy owner is authorised to change the parameters of the
/// ccv module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
#[allow(deprecated)]
fn check_ccv_update_msg_params(
    deps: Deps,
    strategy: Strategy,
    proposal: ProposalExecuteMessage,
) -> Result<(), ContractError> {
    let msg_update_params =
        serde_json_wasm::from_str::<consumer::v1::MsgUpdateParams>(proposal.message.as_str())?
            .params
            .ok_or(ContractError::Unauthorized {})?;

    let ccv_update_param_permission = strategy
        .get_ccv_update_param_permission()
        .ok_or(ContractError::Unauthorized {})?;

    let ccv_params = get_ccv_params(deps)?.params.unwrap_or_default();

    // never allow to change 'enabled'
    if ccv_params.enabled != msg_update_params.enabled {
        return Err(ContractError::Unauthorized {});
    }

    if ccv_params.blocks_per_distribution_transmission
        != msg_update_params.blocks_per_distribution_transmission
        && !ccv_update_param_permission.blocks_per_distribution_transmission
    {
        return Err(ContractError::Unauthorized {});
    }

    if ccv_params.ccv_timeout_period != msg_update_params.ccv_timeout_period
        && !ccv_update_param_permission.ccv_timeout_period
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.consumer_redistribution_fraction
        != msg_update_params.consumer_redistribution_fraction
        && !ccv_update_param_permission.consumer_redistribution_fraction
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.distribution_transmission_channel
        != msg_update_params.distribution_transmission_channel
        && !ccv_update_param_permission.distribution_transmission_channel
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.historical_entries != msg_update_params.historical_entries
        && !ccv_update_param_permission.historical_entries
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.provider_fee_pool_addr_str != msg_update_params.provider_fee_pool_addr_str
        && !ccv_update_param_permission.provider_fee_pool_addr_str
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.provider_reward_denoms != msg_update_params.provider_reward_denoms
        && !ccv_update_param_permission.provider_reward_denoms
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.retry_delay_period != msg_update_params.retry_delay_period
        && !ccv_update_param_permission.retry_delay_period
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.reward_denoms != msg_update_params.reward_denoms
        && !ccv_update_param_permission.reward_denoms
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.soft_opt_out_threshold != msg_update_params.soft_opt_out_threshold
        && !ccv_update_param_permission.soft_opt_out_threshold
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.transfer_timeout_period != msg_update_params.transfer_timeout_period
        && !ccv_update_param_permission.transfer_timeout_period
    {
        return Err(ContractError::Unauthorized {});
    }
    if ccv_params.unbonding_period != msg_update_params.unbonding_period
        && !ccv_update_param_permission.unbonding_period
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Queries the parameters of the ccv module.
pub fn get_ccv_params(deps: Deps) -> StdResult<consumer::v1::QueryParamsResponse> {
    let ccv_querier = consumer::v1::ConsumerQuerier::new(&deps.querier);
    ccv_querier.query_params()
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
