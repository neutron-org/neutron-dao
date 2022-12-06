#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
// use cw_controllers::ClaimsResponse;
use cwd_interface::voting::{TotalPowerAtHeightResponse, VotingPowerAtHeightResponse};
use cwd_interface::{voting, Admin};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, DAO};

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-voting-registry";
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

    let voting_vault = deps.api.addr_validate(&msg.voting_vault)?;

    let config = Config {
        owner,
        manager,
        voting_vaults: vec![voting_vault],
    };

    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute(
            "owner",
            config
                .owner
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        )
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
        ExecuteMsg::AddVotingVault {
            new_voting_vault_contract,
        } => execute_add_voting_vault(deps, env, info, new_voting_vault_contract),
        ExecuteMsg::RemoveVotingVault {
            old_voting_vault_contract,
        } => execute_remove_voting_vault(deps, env, info, old_voting_vault_contract),
        ExecuteMsg::UpdateConfig { owner, manager } => {
            execute_update_config(deps, info, owner, manager)
        }
    }
}

pub fn execute_add_voting_vault(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    new_voting_vault_contact: String,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    let new_voting_vault = deps.api.addr_validate(&new_voting_vault_contact)?;
    if !config.voting_vaults.contains(&new_voting_vault) {
        config.voting_vaults.push(new_voting_vault);
        CONFIG.save(deps.storage, &config)?;
    } else {
        return Err(ContractError::VotingVaultAlreadyExists {});
    }

    Ok(Response::new())
}

pub fn execute_remove_voting_vault(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    old_voting_vault_contact: String,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if config.voting_vaults.len() == 1 {
        Err(ContractError::RemoveLastVault {})
    }

    let new_voting_vault = deps.api.addr_validate(&old_voting_vault_contact)?;
    if config.voting_vaults.contains(&new_voting_vault) {
        config
            .voting_vaults
            .retain(|value| *value != old_voting_vault_contact);
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::new())
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Option<String>,
    new_manager: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if Some(info.sender.clone()) != config.owner && Some(info.sender.clone()) != config.manager {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = new_owner
        .map(|new_owner| deps.api.addr_validate(&new_owner))
        .transpose()?;
    let new_manager = new_manager
        .map(|new_manager| deps.api.addr_validate(&new_manager))
        .transpose()?;

    if Some(info.sender) != config.owner && new_owner != config.owner {
        return Err(ContractError::OnlyOwnerCanChangeOwner {});
    };

    config.owner = new_owner;
    config.manager = new_manager;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute(
            "owner",
            config
                .owner
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        )
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
        QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::VotingVaults {} => query_voting_vaults(deps),
    }
}

pub fn query_voting_vaults(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    to_binary(&config.voting_vaults)
}

pub fn query_voting_power_at_height(
    deps: Deps,
    _env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    let voting_vaults = CONFIG.load(deps.storage)?.voting_vaults;
    // let addr = deps.api.addr_validate(&address)?;
    let mut total_power = VotingPowerAtHeightResponse {
        power: Default::default(),
        height: Default::default(),
    };
    for vault in voting_vaults.iter() {
        let total_power_single_vault: VotingPowerAtHeightResponse = deps.querier.query_wasm_smart(
            vault,
            &voting::Query::VotingPowerAtHeight {
                height,
                address: address.clone(),
            },
        )?;
        total_power.power += total_power_single_vault.power;
        total_power.height = total_power_single_vault.height;
    }

    Ok(total_power)
}

pub fn query_total_power_at_height(
    deps: Deps,
    _env: Env,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let voting_vaults = CONFIG.load(deps.storage)?.voting_vaults;

    let mut total_power: TotalPowerAtHeightResponse = TotalPowerAtHeightResponse {
        power: Default::default(),
        height: Default::default(),
    };
    for vault in voting_vaults.iter() {
        let total_power_single_vault: TotalPowerAtHeightResponse = deps
            .querier
            .query_wasm_smart(vault, &voting::Query::TotalPowerAtHeight { height })?;
        total_power.power += total_power_single_vault.power;
        total_power.height = total_power_single_vault.height;
    }

    Ok(total_power)
}

pub fn query_info(deps: Deps) -> StdResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    to_binary(&voting::InfoResponse { info })
}

pub fn query_dao(deps: Deps) -> StdResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    to_binary(&dao)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
