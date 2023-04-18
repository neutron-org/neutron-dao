#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cwd_interface::voting::{TotalPowerAtHeightResponse, VotingPowerAtHeightResponse};
use cwd_interface::Admin;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, DAO, DESCRIPTION};

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-vesting-vault";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = match msg.owner {
        Admin::Address { addr } => deps.api.addr_validate(addr.as_str())?,
        Admin::CoreModule {} => info.sender.clone(),
    };
    let manager = msg
        .manager
        .map(|manager| deps.api.addr_validate(&manager))
        .transpose()?;

    let vesting_contract_address = deps.api.addr_validate(&msg.vesting_contract_address)?;

    let config = Config {
        vesting_contract_address,
        description: msg.description,
        owner,
        manager,
    };

    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("description", config.description)
        .add_attribute("vesting_contract_address", config.vesting_contract_address)
        .add_attribute("owner", config.owner)
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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            vesting_contract_address,
            owner,
            manager,
            description,
        } => execute_update_config(
            deps,
            info,
            vesting_contract_address,
            owner,
            manager,
            description,
        ),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_vesting_contract_address: Option<String>,
    new_owner: String,
    new_manager: Option<String>,
    new_description: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner && Some(info.sender.clone()) != config.manager {
        return Err(ContractError::Unauthorized {});
    }

    let new_vesting_contract_address = new_vesting_contract_address
        .map(|new_vesting_contract_address| deps.api.addr_validate(&new_vesting_contract_address))
        .transpose()?;

    let new_owner = deps.api.addr_validate(&new_owner)?;
    let new_manager = new_manager
        .map(|new_manager| deps.api.addr_validate(&new_manager))
        .transpose()?;

    if info.sender != config.owner && new_owner != config.owner {
        return Err(ContractError::OnlyOwnerCanChangeOwner {});
    };

    config.owner = new_owner;
    config.manager = new_manager;
    if let Some(description) = new_description {
        config.description = description;
    }
    if let Some(new_vesting_contract_address) = new_vesting_contract_address {
        config.vesting_contract_address = new_vesting_contract_address;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute("vesting_contract_address", config.vesting_contract_address)
        .add_attribute("owner", config.owner)
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
        QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
    }
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    let height = height.unwrap_or(env.block.height);

    let config = CONFIG.load(deps.storage)?;

    let unclaimed_amount: Uint128 = deps.querier.query_wasm_smart(
        config.vesting_contract_address,
        &vesting_base::msg::QueryMsg::HistoricalExtension {
            msg: vesting_base::msg::QueryMsgHistorical::UnclaimedAmountAtHeight { address, height },
        },
    )?;

    Ok(VotingPowerAtHeightResponse {
        power: unclaimed_amount,
        height,
    })
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let height = height.unwrap_or(env.block.height);

    let config = CONFIG.load(deps.storage)?;

    let unclaimed_amount_total: Uint128 = deps.querier.query_wasm_smart(
        config.vesting_contract_address,
        &vesting_base::msg::QueryMsg::HistoricalExtension {
            msg: vesting_base::msg::QueryMsgHistorical::UnclaimedTotalAmountAtHeight { height },
        },
    )?;

    Ok(TotalPowerAtHeightResponse {
        power: unclaimed_amount_total,
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
