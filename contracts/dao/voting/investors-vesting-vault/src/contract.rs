#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, DAO};

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-investors-vesting-vault";
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
    let vesting_contract_address = deps.api.addr_validate(&msg.vesting_contract_address)?;

    let config = Config {
        vesting_contract_address,
        description: msg.description,
        owner,
        name: msg.name,
    };

    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("description", config.description)
        .add_attribute("vesting_contract_address", config.vesting_contract_address)
        .add_attribute("owner", config.owner))
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
            vesting_contract_address,
            owner,
            description,
            name,
        } => execute_update_config(
            deps,
            info,
            vesting_contract_address,
            owner,
            description,
            name,
        ),
    }
}

pub fn execute_bond(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    Err(ContractError::BondingDisabled {})
}

pub fn execute_unbond(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    Err(ContractError::DirectUnbondingDisabled {})
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_vesting_contract_address: Option<String>,
    new_owner: Option<String>,
    new_description: Option<String>,
    new_name: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_vesting_contract_address = new_vesting_contract_address
        .map(|new_vesting_contract_address| deps.api.addr_validate(&new_vesting_contract_address))
        .transpose()?;

    let new_owner = new_owner
        .map(|new_owner| deps.api.addr_validate(&new_owner))
        .transpose()?;

    if let Some(owner) = new_owner {
        config.owner = owner;
    }

    if let Some(name) = new_name {
        config.name = name;
    }

    if let Some(description) = new_description {
        config.description = description;
    }
    if let Some(new_vesting_contract_address) = new_vesting_contract_address {
        config.vesting_contract_address = new_vesting_contract_address;
    }

    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute("vesting_contract_address", config.vesting_contract_address)
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
        QueryMsg::Name {} => query_name(deps),
        QueryMsg::Description {} => query_description(deps),
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::BondingStatus { height, address } => {
            to_binary(&query_bonding_status(deps, env, height, address)?)
        }
        QueryMsg::ListBonders { start_after, limit } => {
            query_list_bonders(deps, env, start_after, limit)
        }
    }
}

pub fn query_list_bonders(
    _deps: Deps,
    _env: Env,
    _start_after: Option<String>,
    _limit: Option<u32>,
) -> StdResult<Binary> {
    Err(StdError::generic_err(format!(
        "{}",
        ContractError::BondingDisabled {}
    )))
}

pub fn query_bonding_status(
    _deps: Deps,
    _env: Env,
    _height: Option<u64>,
    _address: String,
) -> StdResult<BondingStatusResponse> {
    Err(StdError::generic_err(format!(
        "{}",
        ContractError::BondingDisabled {}
    )))
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

pub fn query_name(deps: Deps) -> StdResult<Binary> {
    let config: Config = CONFIG.load(deps.storage)?;
    to_binary(&config.name)
}

pub fn query_description(deps: Deps) -> StdResult<Binary> {
    let config: Config = CONFIG.load(deps.storage)?;
    to_binary(&config.description)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
