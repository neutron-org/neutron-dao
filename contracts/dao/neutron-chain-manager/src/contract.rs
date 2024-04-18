use crate::cron_module_param_types::{
    ParamsRequestCron, ParamsResponseCron, PARAMS_QUERY_PATH_CRON,
};
use crate::permission::{match_permission_type, Permission, PermissionType, Validator};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::proto_types::neutron::cron::QueryParamsRequest;
use neutron_sdk::stargate::aux::make_stargate_query;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::PERMISSIONS;

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

    PERMISSIONS.save(
        deps.storage,
        (msg.initial_address.clone(), PermissionType::AllowAll),
        &Permission::AllowAll,
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("init_allow_all_address", msg.initial_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::AddPermissions {
            address,
            permissions,
        } => execute_add_permissions(deps, info, address, permissions),
        ExecuteMsg::RemovePermissions { address } => execute_remove_strategy(deps, info, address),
        ExecuteMsg::ExecuteMessages { messages } => execute_execute_messages(deps, info, messages),
    }
}

pub fn execute_add_permissions(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
    permissions: Vec<Permission>,
) -> Result<Response<NeutronMsg>, ContractError> {
    is_authorized(deps.as_ref(), info.sender.clone())?;

    // We add the new strategy, and then we check that it did not replace
    // the only existing ALLOW_ALL strategy.
    for p in &permissions {
        PERMISSIONS.save(deps.storage, (address.clone(), p.to_owned().into()), p)?;
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

    // First we remove all the permissions, then we check that it was not the only
    // ALLOW_ALL permission we had.
    let permission_types = PERMISSIONS
        .prefix(address.clone())
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<Result<Vec<PermissionType>, StdError>>()?;
    for permission in permission_types {
        PERMISSIONS.remove(deps.storage, (address.clone(), permission))
    }

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
    let response = Response::new()
        .add_attribute("action", "execute_execute_messages")
        .add_attribute("address", info.sender.clone());

    // check the sender is AllowAll account
    if PERMISSIONS.has(
        deps.storage,
        (info.sender.clone(), PermissionType::AllowAll),
    ) {
        return Ok(response
            .add_attribute("strategy", "allow_all")
            .add_messages(messages));
    };

    // сопоставляем космос сообщения с типами требуемых разрешений
    let permissions_types: Vec<(PermissionType, Validator)> = messages
        .iter()
        .map(match_permission_type)
        .collect::<Result<Vec<(PermissionType, Validator)>, ContractError>>()?;

    for (p, v) in permissions_types {
        let permission = PERMISSIONS.load(deps.storage, (info.sender.clone(), p))?;
        v.validate(deps.as_ref(), permission)?
    }

    Ok(response
        .add_attribute("strategy", "allow_only")
        .add_messages(messages))
}

fn is_authorized(deps: Deps, address: Addr) -> Result<(), ContractError> {
    if PERMISSIONS.has(deps.storage, (address, PermissionType::AllowAll)) {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

/// This function returns true if there is no more allow_all strategies left.
fn no_admins_left(deps: Deps) -> Result<bool, ContractError> {
    let not_found: bool = !PERMISSIONS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<Result<Vec<(Addr, PermissionType)>, _>>()?
        .into_iter()
        .any(|(_, perm_type)| perm_type == PermissionType::AllowAll);
    Ok(not_found)
}

/// Queries the parameters of the cron module.
pub fn get_cron_params(deps: Deps, req: ParamsRequestCron) -> StdResult<ParamsResponseCron> {
    make_stargate_query(deps, PARAMS_QUERY_PATH_CRON, QueryParamsRequest::from(req))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Permissions {} => to_json_binary(&query_strategies(deps)?),
    }
}

/// No pagination is added because it's unlikely that there is going
/// to be more than 10 strategies.
pub fn query_strategies(deps: Deps) -> StdResult<Vec<(Addr, Permission)>> {
    // TODO group by addresses
    let all_permissions: Vec<(Addr, Permission)> = PERMISSIONS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<Result<Vec<((Addr, PermissionType), Permission)>, _>>()?
        .into_iter()
        .map(|((a, _), p)| (a, p))
        .collect();
    Ok(all_permissions)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
