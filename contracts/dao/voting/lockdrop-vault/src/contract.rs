#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use cwd_interface::Admin;

use crate::state::{CONFIG, DAO};
use neutron_lockdrop_vault::error::ContractError;
use neutron_lockdrop_vault::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use neutron_lockdrop_vault::types::Config;

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-lockdrop-vault";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .as_ref()
        .map(|owner| match owner {
            Admin::Address { addr } => deps.api.addr_validate(addr),
            Admin::CoreModule {} => Ok(info.sender.clone()),
        })
        .transpose()?;
    let manager = msg
        .manager
        .map(|manager| deps.api.addr_validate(&manager))
        .transpose()?;

    let config = Config {
        description: msg.description,
        lockdrop_contract: deps.api.addr_validate(&msg.lockdrop_contract)?,
        owner,
        manager,
    };
    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("description", config.description)
        .add_attribute(
            "owner",
            config
                .owner
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        )
        .add_attribute("lockdrop_contract", config.lockdrop_contract)
        .add_attribute(
            "manager",
            config
                .manager
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bond {} => execute_bond(deps, env, info),
        ExecuteMsg::Unbond { amount } => execute_unbond(deps, env, info, amount),
        ExecuteMsg::UpdateConfig {
            owner,
            lockdrop_contract,
            manager,
            description,
        } => execute_update_config(deps, info, owner, lockdrop_contract, manager, description),
    }
}

pub fn execute_bond(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn execute_unbond(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Option<String>,
    new_lockdrop_contract: String,
    new_manager: Option<String>,
    new_description: String,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if Some(info.sender.clone()) != config.owner && Some(info.sender.clone()) != config.manager {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = new_owner
        .map(|new_owner| deps.api.addr_validate(&new_owner))
        .transpose()?;
    let new_lockdrop_contract = deps.api.addr_validate(&new_lockdrop_contract)?;
    let new_manager = new_manager
        .map(|new_manager| deps.api.addr_validate(&new_manager))
        .transpose()?;

    if Some(info.sender.clone()) != config.owner && new_owner != config.owner {
        return Err(ContractError::OnlyOwnerCanChangeOwner {});
    };
    if Some(info.sender) != config.owner
        && new_lockdrop_contract != config.clone().lockdrop_contract
    {
        return Err(ContractError::OnlyOwnerCanChangeLockdropContract {});
    };

    config.owner = new_owner;
    config.lockdrop_contract = new_lockdrop_contract;
    config.manager = new_manager;
    config.description = new_description;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute(
            "owner",
            config
                .owner
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        )
        .add_attribute("lockdrop_contract", config.lockdrop_contract)
        .add_attribute(
            "manager",
            config
                .manager
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VotingPowerAtHeight { address, height } => {
            to_binary(&query_voting_power_at_height(deps, env, address, height)?)
        }
        QueryMsg::TotalPowerAtHeight { height } => {
            to_binary(&query_total_power_at_height(deps, env, height)?)
        }
        QueryMsg::Info {} => query_info(deps),
        QueryMsg::Dao {} => query_dao(deps),
        QueryMsg::Description {} => query_description(deps),
        QueryMsg::GetConfig {} => query_config(deps),
        QueryMsg::ListBonders { start_after, limit } => {
            query_list_bonders(deps, start_after, limit)
        }
        QueryMsg::BondingStatus { height, address } => {
            to_binary(&query_bonding_status(deps, env, height, address)?)
        }
    }
}

pub fn query_voting_power_at_height(
    _deps: Deps,
    _env: Env,
    _address: String,
    _height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    // TODO: implement once the lockdrop contract is implemented.
    unimplemented!()
}

pub fn query_total_power_at_height(
    _deps: Deps,
    _env: Env,
    _height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    // TODO: implement once the lockdrop contract is implemented.
    unimplemented!()
}

pub fn query_info(deps: Deps) -> StdResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    to_binary(&cwd_interface::voting::InfoResponse { info })
}

pub fn query_dao(deps: Deps) -> StdResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    to_binary(&dao)
}

pub fn query_description(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&config.description)
}

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&config)
}

pub fn query_list_bonders(
    _deps: Deps,
    _start_after: Option<String>,
    _limit: Option<u32>,
) -> StdResult<Binary> {
    // TODO: implement once the lockdrop contract is implemented.
    unimplemented!()
}

pub fn query_bonding_status(
    _deps: Deps,
    env: Env,
    height: Option<u64>,
    _address: String,
) -> StdResult<BondingStatusResponse> {
    let height = height.unwrap_or(env.block.height);
    Ok(BondingStatusResponse {
        unbondable_abount: Uint128::zero(),
        bonding_enabled: false,
        height,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
