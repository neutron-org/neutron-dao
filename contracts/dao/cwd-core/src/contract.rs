#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_utils::{parse_instantiate_response_data, Duration, ParseReplyError};

use cw_paginate::{paginate_map, paginate_map_values};
use cwd_interface::{voting, ModuleInstantiateInfo};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_subdao_core::types::SubDao;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InitialItem, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{DumpStateResponse, GetItemResponse, PauseInfoResponse};
use crate::state::{
    Config, ProposalModule, ProposalModuleStatus, ACTIVE_PROPOSAL_MODULE_COUNT, CONFIG, ITEMS,
    PAUSED, PROPOSAL_MODULES, SUBDAO_LIST, TOTAL_PROPOSAL_MODULE_COUNT, VOTING_REGISTRY_MODULE,
};

pub(crate) const CONTRACT_NAME: &str = "crates.io:cwd-core";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PROPOSAL_MODULE_REPLY_ID: u64 = 0;
const VOTE_MODULE_INSTANTIATE_REPLY_ID: u64 = 1;
const VOTE_MODULE_UPDATE_REPLY_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        name: msg.name,
        description: msg.description,
        dao_uri: msg.dao_uri,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    let vote_module_msg = msg
        .voting_registry_module_instantiate_info
        .into_wasm_msg(env.contract.address.clone());
    let vote_module_msg: SubMsg<NeutronMsg> =
        SubMsg::reply_on_success(vote_module_msg, VOTE_MODULE_INSTANTIATE_REPLY_ID);

    let proposal_module_msgs: Vec<SubMsg<NeutronMsg>> = msg
        .proposal_modules_instantiate_info
        .into_iter()
        .map(|info| info.into_wasm_msg(env.contract.address.clone()))
        .map(|wasm| SubMsg::reply_on_success(wasm, PROPOSAL_MODULE_REPLY_ID))
        .collect();
    if proposal_module_msgs.is_empty() {
        return Err(ContractError::NoActiveProposalModules {});
    }

    for InitialItem { key, value } in msg.initial_items.unwrap_or_default() {
        ITEMS.save(deps.storage, key, &value)?;
    }

    TOTAL_PROPOSAL_MODULE_COUNT.save(deps.storage, &0)?;
    ACTIVE_PROPOSAL_MODULE_COUNT.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("sender", info.sender)
        .add_submessage(vote_module_msg)
        .add_submessages(proposal_module_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    // No actions can be performed while the DAO is paused.
    if let Some(expiration) = PAUSED.may_load(deps.storage)? {
        if !expiration.is_expired(&env.block) {
            return Err(ContractError::Paused {});
        }
    }

    match msg {
        ExecuteMsg::ExecuteProposalHook { msgs } => {
            execute_proposal_hook(deps.as_ref(), info.sender, msgs)
        }
        ExecuteMsg::Pause { duration } => execute_pause(deps, env, info.sender, duration),
        ExecuteMsg::RemoveItem { key } => execute_remove_item(deps, env, info.sender, key),
        ExecuteMsg::SetItem { key, addr } => execute_set_item(deps, env, info.sender, key, addr),
        ExecuteMsg::UpdateConfig { config } => {
            execute_update_config(deps, env, info.sender, config)
        }
        ExecuteMsg::UpdateVotingModule { module } => {
            execute_update_voting_module(env, info.sender, module)
        }
        ExecuteMsg::UpdateProposalModules { to_add, to_disable } => {
            execute_update_proposal_modules(deps, env, info.sender, to_add, to_disable)
        }
        ExecuteMsg::UpdateSubDaos { to_add, to_remove } => {
            execute_update_sub_daos_list(deps, env, info.sender, to_add, to_remove)
        }
    }
}

pub fn execute_pause(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    pause_duration: Duration,
) -> Result<Response<NeutronMsg>, ContractError> {
    if sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let until = pause_duration.after(&env.block);

    PAUSED.save(deps.storage, &until)?;

    Ok(Response::new()
        .add_attribute("action", "execute_pause")
        .add_attribute("sender", sender)
        .add_attribute("until", until.to_string()))
}

pub fn execute_proposal_hook(
    deps: Deps,
    sender: Addr,
    msgs: Vec<CosmosMsg<NeutronMsg>>,
) -> Result<Response<NeutronMsg>, ContractError> {
    let module = PROPOSAL_MODULES
        .may_load(deps.storage, sender.clone())?
        .ok_or(ContractError::Unauthorized {})?;

    // Check that the message has come from an active module
    if module.status != ProposalModuleStatus::Enabled {
        return Err(ContractError::ModuleDisabledCannotExecute { address: sender });
    }

    Ok(Response::default()
        .add_attribute("action", "execute_proposal_hook")
        .add_messages(msgs))
}

pub fn execute_update_config(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    config: Config,
) -> Result<Response<NeutronMsg>, ContractError> {
    if env.contract.address != sender {
        return Err(ContractError::Unauthorized {});
    }

    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    // We incur some gas costs by having the config's fields in the
    // response. This has the benefit that it makes it reasonably
    // simple to ask "when did this field in the config change" by
    // running something like `junod query txs --events
    // 'wasm._contract_address=core&wasm.name=name'`.
    Ok(Response::default()
        .add_attribute("action", "execute_update_config")
        .add_attribute("name", config.name)
        .add_attribute("description", config.description)
        .add_attribute(
            "dao_uri",
            config.dao_uri.unwrap_or_else(|| String::from("None")),
        ))
}

pub fn execute_update_voting_module(
    env: Env,
    sender: Addr,
    module: ModuleInstantiateInfo,
) -> Result<Response<NeutronMsg>, ContractError> {
    if env.contract.address != sender {
        return Err(ContractError::Unauthorized {});
    }

    let wasm = module.into_wasm_msg(env.contract.address);
    let submessage = SubMsg::reply_on_success(wasm, VOTE_MODULE_UPDATE_REPLY_ID);

    Ok(Response::default()
        .add_attribute("action", "execute_update_voting_module")
        .add_submessage(submessage))
}

pub fn execute_update_proposal_modules(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    to_add: Vec<ModuleInstantiateInfo>,
    to_disable: Vec<String>,
) -> Result<Response<NeutronMsg>, ContractError> {
    if env.contract.address != sender {
        return Err(ContractError::Unauthorized {});
    }

    let disable_count = to_disable.len() as u32;
    for addr in to_disable {
        let addr = deps.api.addr_validate(&addr)?;
        let mut module = PROPOSAL_MODULES
            .load(deps.storage, addr.clone())
            .map_err(|_| ContractError::ProposalModuleDoesNotExist {
                address: addr.clone(),
            })?;

        if module.status == ProposalModuleStatus::Disabled {
            return Err(ContractError::ModuleAlreadyDisabled {
                address: module.address,
            });
        }

        module.status = ProposalModuleStatus::Disabled {};
        PROPOSAL_MODULES.save(deps.storage, addr, &module)?;
    }

    // If disabling this module will cause there to be no active modules, return error.
    // We don't check the active count before disabling because there may erroneously be
    // modules in to_disable which are already disabled.
    ACTIVE_PROPOSAL_MODULE_COUNT.update(deps.storage, |count| {
        if count <= disable_count && to_add.is_empty() {
            return Err(ContractError::NoActiveProposalModules {});
        }
        Ok(count - disable_count)
    })?;

    let to_add: Vec<SubMsg<NeutronMsg>> = to_add
        .into_iter()
        .map(|info| info.into_wasm_msg(env.contract.address.clone()))
        .map(|wasm| SubMsg::reply_on_success(wasm, PROPOSAL_MODULE_REPLY_ID))
        .collect();

    Ok(Response::default()
        .add_attribute("action", "execute_update_proposal_modules")
        .add_submessages(to_add))
}

pub fn execute_set_item(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    key: String,
    value: String,
) -> Result<Response<NeutronMsg>, ContractError> {
    if env.contract.address != sender {
        return Err(ContractError::Unauthorized {});
    }

    ITEMS.save(deps.storage, key.clone(), &value)?;
    Ok(Response::default()
        .add_attribute("action", "execute_set_item")
        .add_attribute("key", key)
        .add_attribute("addr", value))
}

pub fn execute_remove_item(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    key: String,
) -> Result<Response<NeutronMsg>, ContractError> {
    if env.contract.address != sender {
        return Err(ContractError::Unauthorized {});
    }

    if ITEMS.has(deps.storage, key.clone()) {
        ITEMS.remove(deps.storage, key.clone());
        Ok(Response::default()
            .add_attribute("action", "execute_remove_item")
            .add_attribute("key", key))
    } else {
        Err(ContractError::KeyMissing {})
    }
}

pub fn execute_update_sub_daos_list(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    to_add: Vec<SubDao>,
    to_remove: Vec<String>,
) -> Result<Response<NeutronMsg>, ContractError> {
    if env.contract.address != sender {
        return Err(ContractError::Unauthorized {});
    }

    for addr in to_remove {
        let addr = deps.api.addr_validate(&addr)?;
        SUBDAO_LIST.remove(deps.storage, &addr);
    }

    for subdao in to_add {
        let addr = deps.api.addr_validate(&subdao.addr)?;
        SUBDAO_LIST.save(deps.storage, &addr, &subdao.charter)?;
    }

    Ok(Response::default()
        .add_attribute("action", "execute_update_sub_daos_list")
        .add_attribute("sender", sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::DumpState {} => query_dump_state(deps, env),
        QueryMsg::GetItem { key } => query_get_item(deps, key),
        QueryMsg::Info {} => query_info(deps),
        QueryMsg::ListItems { start_after, limit } => query_list_items(deps, start_after, limit),
        QueryMsg::PauseInfo {} => query_paused(deps, env),
        QueryMsg::ProposalModules { start_after, limit } => {
            query_proposal_modules(deps, start_after, limit)
        }
        QueryMsg::TotalPowerAtHeight { height } => query_total_power_at_height(deps, height),
        QueryMsg::VotingModule {} => query_voting_module(deps),
        QueryMsg::VotingPowerAtHeight { address, height } => {
            query_voting_power_at_height(deps, address, height)
        }
        QueryMsg::ActiveProposalModules { start_after, limit } => {
            query_active_proposal_modules(deps, start_after, limit)
        }
        QueryMsg::ListSubDaos { start_after, limit } => {
            query_list_sub_daos(deps, start_after, limit)
        }
        QueryMsg::GetSubDao { address } => query_sub_dao(deps, address),
        QueryMsg::DaoURI {} => query_dao_uri(deps),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
}

pub fn query_voting_module(deps: Deps) -> StdResult<Binary> {
    let voting_module = VOTING_REGISTRY_MODULE.load(deps.storage)?;
    to_json_binary(&voting_module)
}

pub fn query_proposal_modules(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    // This query is will run out of gas due to the size of the
    // returned message before it runs out of compute so taking a
    // limit here is still nice. As removes happen in constant time
    // the contract is still recoverable if too many items end up in
    // here.
    //
    // Further, as the `range` method on a map returns an iterator it
    // is possible (though implementation dependent) that new keys are
    // loaded on demand as the iterator is moved. Should this be the
    // case we are only paying for what we need here.
    //
    // Even if this does lock up one can determine the existing
    // proposal modules by looking at past transactions on chain.
    to_json_binary(&paginate_map_values(
        deps,
        &PROPOSAL_MODULES,
        start_after
            .map(|s| deps.api.addr_validate(&s))
            .transpose()?,
        limit,
        cosmwasm_std::Order::Ascending,
    )?)
}

/// Note: this is not gas efficient as we need to potentially visit all modules in order to
/// filter out the modules with active status.
pub fn query_active_proposal_modules(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let values = paginate_map_values(
        deps,
        &PROPOSAL_MODULES,
        start_after
            .map(|s| deps.api.addr_validate(&s))
            .transpose()?,
        None,
        cosmwasm_std::Order::Ascending,
    )?;

    let limit = limit.unwrap_or(values.len() as u32);

    to_json_binary::<Vec<ProposalModule>>(
        &values
            .into_iter()
            .filter(|module: &ProposalModule| module.status == ProposalModuleStatus::Enabled)
            .take(limit as usize)
            .collect(),
    )
}

fn get_pause_info(deps: Deps, env: Env) -> StdResult<PauseInfoResponse> {
    Ok(match PAUSED.may_load(deps.storage)? {
        Some(expiration) => {
            if expiration.is_expired(&env.block) {
                PauseInfoResponse::Unpaused {}
            } else {
                PauseInfoResponse::Paused { expiration }
            }
        }
        None => PauseInfoResponse::Unpaused {},
    })
}

pub fn query_paused(deps: Deps, env: Env) -> StdResult<Binary> {
    to_json_binary(&get_pause_info(deps, env)?)
}

pub fn query_dump_state(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let voting_registry_module = VOTING_REGISTRY_MODULE.load(deps.storage)?;
    let proposal_modules = PROPOSAL_MODULES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|kv| Ok(kv?.1))
        .collect::<StdResult<Vec<ProposalModule>>>()?;
    let pause_info = get_pause_info(deps, env)?;
    let version = get_contract_version(deps.storage)?;
    let active_proposal_module_count = ACTIVE_PROPOSAL_MODULE_COUNT.load(deps.storage)?;
    let total_proposal_module_count = TOTAL_PROPOSAL_MODULE_COUNT.load(deps.storage)?;
    to_json_binary(&DumpStateResponse {
        config,
        version,
        pause_info,
        proposal_modules,
        voting_module: voting_registry_module,
        active_proposal_module_count,
        total_proposal_module_count,
    })
}

pub fn query_voting_power_at_height(
    deps: Deps,
    address: String,
    height: Option<u64>,
) -> StdResult<Binary> {
    let voting_registry_module = VOTING_REGISTRY_MODULE.load(deps.storage)?;
    let voting_power: voting::VotingPowerAtHeightResponse = deps.querier.query_wasm_smart(
        voting_registry_module,
        &voting::Query::VotingPowerAtHeight { height, address },
    )?;
    to_json_binary(&voting_power)
}

pub fn query_total_power_at_height(deps: Deps, height: Option<u64>) -> StdResult<Binary> {
    let voting_registry_module = VOTING_REGISTRY_MODULE.load(deps.storage)?;
    let total_power: voting::TotalPowerAtHeightResponse = deps.querier.query_wasm_smart(
        voting_registry_module,
        &voting::Query::TotalPowerAtHeight { height },
    )?;
    to_json_binary(&total_power)
}

pub fn query_get_item(deps: Deps, item: String) -> StdResult<Binary> {
    let item = ITEMS.may_load(deps.storage, item)?;
    to_json_binary(&GetItemResponse { item })
}

pub fn query_info(deps: Deps) -> StdResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    to_json_binary(&cwd_interface::voting::InfoResponse { info })
}

pub fn query_list_items(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    to_json_binary(&paginate_map(
        deps,
        &ITEMS,
        start_after,
        limit,
        cosmwasm_std::Order::Descending,
    )?)
}

pub fn query_list_sub_daos(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start_at = start_after
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?;

    let subdaos = cw_paginate::paginate_map(
        deps,
        &SUBDAO_LIST,
        start_at.as_ref(),
        limit,
        cosmwasm_std::Order::Ascending,
    )?;

    let subdaos: Vec<SubDao> = subdaos
        .into_iter()
        .map(|(address, charter)| SubDao {
            addr: address.into_string(),
            charter,
        })
        .collect();

    to_json_binary(&subdaos)
}

pub fn query_sub_dao(deps: Deps, address: String) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&address)?;
    let item = SUBDAO_LIST.may_load(deps.storage, &addr)?;
    match item {
        None => Err(StdError::generic_err("SubDao not found")),
        Some(charter) => to_json_binary(&SubDao {
            addr: address,
            charter,
        }),
    }
}

pub fn query_dao_uri(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config.dao_uri)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response<NeutronMsg>, ContractError> {
    match msg.id {
        PROPOSAL_MODULE_REPLY_ID => {
            let res = parse_instantiate_response_data(
                &msg.result
                    .into_result()
                    .map_err(ParseReplyError::SubMsgFailure)?
                    .msg_responses[0]
                    .value,
            )?;
            let prop_module_addr = deps.api.addr_validate(&res.contract_address)?;
            let total_module_count = TOTAL_PROPOSAL_MODULE_COUNT.load(deps.storage)?;

            let prefix = derive_proposal_module_prefix(total_module_count as usize)?;
            let prop_module = ProposalModule {
                address: prop_module_addr.clone(),
                status: ProposalModuleStatus::Enabled,
                prefix,
            };

            PROPOSAL_MODULES.save(deps.storage, prop_module_addr, &prop_module)?;

            // Save active and total proposal module counts.
            ACTIVE_PROPOSAL_MODULE_COUNT
                .update::<_, StdError>(deps.storage, |count| Ok(count + 1))?;
            TOTAL_PROPOSAL_MODULE_COUNT.save(deps.storage, &(total_module_count + 1))?;

            Ok(Response::default().add_attribute("prop_module".to_string(), res.contract_address))
        }

        VOTE_MODULE_INSTANTIATE_REPLY_ID => {
            let res = parse_instantiate_response_data(
                &msg.result
                    .into_result()
                    .map_err(ParseReplyError::SubMsgFailure)?
                    .msg_responses[0]
                    .value,
            )?;
            let voting_registry_addr = deps.api.addr_validate(&res.contract_address)?;
            let current = VOTING_REGISTRY_MODULE.may_load(deps.storage)?;

            // Make sure a bug in instantiation isn't causing us to
            // make more than one voting module.
            if current.is_some() {
                return Err(ContractError::MultipleVotingModules {});
            }

            VOTING_REGISTRY_MODULE.save(deps.storage, &voting_registry_addr)?;

            Ok(Response::default().add_attribute("voting_regsitry_module", voting_registry_addr))
        }
        VOTE_MODULE_UPDATE_REPLY_ID => {
            let res = parse_instantiate_response_data(
                &msg.result
                    .into_result()
                    .map_err(ParseReplyError::SubMsgFailure)?
                    .msg_responses[0]
                    .value,
            )?;
            let voting_registry_addr = deps.api.addr_validate(&res.contract_address)?;

            VOTING_REGISTRY_MODULE.save(deps.storage, &voting_registry_addr)?;

            Ok(Response::default().add_attribute("voting_registry_module", voting_registry_addr))
        }
        _ => Err(ContractError::UnknownReplyID {}),
    }
}

pub(crate) fn derive_proposal_module_prefix(mut dividend: usize) -> StdResult<String> {
    dividend += 1;
    // Pre-allocate string
    let mut prefix = String::with_capacity(10);
    loop {
        let remainder = (dividend - 1) % 26;
        dividend = (dividend - remainder) / 26;
        let remainder_str = std::str::from_utf8(&[(remainder + 65) as u8])?.to_owned();
        prefix.push_str(&remainder_str);
        if dividend == 0 {
            break;
        }
    }
    Ok(prefix.chars().rev().collect())
}

#[cfg(test)]
mod test {
    use crate::contract::derive_proposal_module_prefix;
    use std::collections::HashSet;

    #[test]
    fn test_prefix_generation() {
        assert_eq!("A", derive_proposal_module_prefix(0).unwrap());
        assert_eq!("B", derive_proposal_module_prefix(1).unwrap());
        assert_eq!("C", derive_proposal_module_prefix(2).unwrap());
        assert_eq!("AA", derive_proposal_module_prefix(26).unwrap());
        assert_eq!("AB", derive_proposal_module_prefix(27).unwrap());
        assert_eq!("BA", derive_proposal_module_prefix(26 * 2).unwrap());
        assert_eq!("BB", derive_proposal_module_prefix(26 * 2 + 1).unwrap());
        assert_eq!("CA", derive_proposal_module_prefix(26 * 3).unwrap());
        assert_eq!("JA", derive_proposal_module_prefix(26 * 10).unwrap());
        assert_eq!("YA", derive_proposal_module_prefix(26 * 25).unwrap());
        assert_eq!("ZA", derive_proposal_module_prefix(26 * 26).unwrap());
        assert_eq!("ZZ", derive_proposal_module_prefix(26 * 26 + 25).unwrap());
        assert_eq!("AAA", derive_proposal_module_prefix(26 * 26 + 26).unwrap());
        assert_eq!("YZA", derive_proposal_module_prefix(26 * 26 * 26).unwrap());
        assert_eq!("ZZ", derive_proposal_module_prefix(26 * 26 + 25).unwrap());
    }

    #[test]
    fn test_prefixes_no_collisions() {
        let mut seen = HashSet::<String>::new();
        for i in 0..25 * 25 * 25 {
            let prefix = derive_proposal_module_prefix(i).unwrap();
            if seen.contains(&prefix) {
                panic!("already seen value")
            }
            seen.insert(prefix);
        }
    }
}
