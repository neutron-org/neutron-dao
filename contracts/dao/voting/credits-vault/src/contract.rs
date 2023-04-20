#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cwd_interface::voting::{TotalPowerAtHeightResponse, VotingPowerAtHeightResponse};

use crate::error::ContractError;
use crate::msg::{CreditsQueryMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, TotalSupplyResponse, CONFIG, DAO, DESCRIPTION};

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-credits-vault";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    let credits_contract_address = deps.api.addr_validate(&msg.credits_contract_address)?;

    let config = Config {
        credits_contract_address,
        description: msg.description,
        owner,
    };

    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("description", config.description)
        .add_attribute("credits_contract_address", config.credits_contract_address)
        .add_attribute("owner", config.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            credits_contract_address,
            owner,
            description,
        } => execute_update_config(deps, info, credits_contract_address, owner, description),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_credits_contract_address: Option<String>,
    new_owner: String,
    new_description: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_credits_contract_address = new_credits_contract_address
        .map(|new_credits_contract_address| deps.api.addr_validate(&new_credits_contract_address))
        .transpose()?;
    let new_owner = deps.api.addr_validate(&new_owner)?;

    config.owner = new_owner;
    if let Some(description) = new_description {
        config.description = description;
    }
    if let Some(new_credits_contract_address) = new_credits_contract_address {
        config.credits_contract_address = new_credits_contract_address;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute("credits_contract_address", config.credits_contract_address)
        .add_attribute("owner", config.owner))
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
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
    }
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    let config = CONFIG.load(deps.storage)?;

    let height = height.unwrap_or(env.block.height);

    let balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.credits_contract_address,
        &CreditsQueryMsg::BalanceAtHeight {
            height: Some(height),
            address,
        },
    )?;

    Ok(VotingPowerAtHeightResponse {
        power: balance.balance,
        height,
    })
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let config = CONFIG.load(deps.storage)?;

    let height = height.unwrap_or(env.block.height);

    let total_supply: TotalSupplyResponse = deps.querier.query_wasm_smart(
        config.credits_contract_address,
        &CreditsQueryMsg::TotalSupplyAtHeight {
            height: Some(height),
        },
    )?;

    Ok(TotalPowerAtHeightResponse {
        power: total_supply.total_supply,
        height,
    })
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
    let description = DESCRIPTION.load(deps.storage)?;
    to_binary(&description)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
