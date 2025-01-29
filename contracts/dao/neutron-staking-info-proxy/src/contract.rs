use cosmwasm_std::{
    entry_point, to_json_binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::error::ContractError::Unauthorized;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:neutron-staking-info-proxy";
const CONTRACT_VERSION: &str = "0.1.0";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    let staking_vault = deps.api.addr_validate(&msg.staking_vault)?;
    let config = Config {
        owner,
        staking_vault,
    };
    config.validate()?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            staking_vault,
        } => update_config(deps, env, info, owner, staking_vault),
        // Updates the stake information for a particular user
        ExecuteMsg::UpdateStake { user } => update_stake(deps, env, info, user),
    }
}

/// Updates configuration parameters for the contract.
/// Only the current owner can call this method.
#[allow(clippy::too_many_arguments)]
fn update_config(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    owner: Option<String>,
    staking_vault: Option<String>,
) -> Result<Response, ContractError> {
    // Load the existing configuration
    let mut config = CONFIG.load(deps.storage)?;

    // Ensure only the contract owner can update the configuration
    if info.sender != config.owner {
        return Err(Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(staking_vault) = staking_vault {
        config.staking_vault = deps.api.addr_validate(&staking_vault)?;
    }

    // Validate updated config and save
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner.to_string()))
}

/// Called by the staking_info_proxy to update a user’s staked amount in this contract’s state.
/// This keeps track of user-level reward data (pending rewards, reward index).
fn update_stake(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    user: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.staking_vault {
        return Err(Unauthorized {});
    }

    Ok(Response::new()
        .add_attribute("action", "update_stake")
        .add_attribute("user", user.clone()))
}

// ----------------------------------------
//  Queries
// ----------------------------------------
#[entry_point]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<cosmwasm_std::Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&query_config(deps)?)?),
    }
}

/// Returns only the config (no state fields).
fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        staking_vault: config.staking_vault.to_string(),
    })
}

// ----------------------------------------
//  Migration
// ----------------------------------------
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
