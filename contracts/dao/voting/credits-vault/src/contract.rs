use crate::error::ContractError;
use crate::msg::{CreditsQueryMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, TotalSupplyResponse, CONFIG, DAO};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};

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

    let airdrop_contract_address = deps.api.addr_validate(&msg.airdrop_contract_address)?;

    let config = Config {
        name: msg.name,
        description: msg.description,
        credits_contract_address,
        owner,
        airdrop_contract_address,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("description", config.description)
        .add_attribute("credits_contract_address", config.credits_contract_address)
        .add_attribute("airdrop_contract_address", config.airdrop_contract_address)
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
            credits_contract_address,
            owner,
            name,
            description,
        } => execute_update_config(
            deps,
            info,
            credits_contract_address,
            owner,
            name,
            description,
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
    new_credits_contract_address: Option<String>,
    new_owner: Option<String>,
    new_name: Option<String>,
    new_description: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = new_owner
        .map(|new_owner| deps.api.addr_validate(&new_owner))
        .transpose()?;

    let new_credits_contract_address = new_credits_contract_address
        .map(|new_credits_contract_address| deps.api.addr_validate(&new_credits_contract_address))
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
    if let Some(new_credits_contract_address) = new_credits_contract_address {
        config.credits_contract_address = new_credits_contract_address;
    }

    config.validate()?;
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
        QueryMsg::Name {} => query_name(deps),
        QueryMsg::Description {} => query_description(deps),
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::ListBonders { start_after, limit } => {
            query_list_bonders(deps, start_after, limit)
        }
        QueryMsg::BondingStatus { height, address } => {
            to_binary(&query_bonding_status(deps, env, height, address)?)
        }
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

    let balance = if address == config.airdrop_contract_address {
        Uint128::zero()
    } else {
        deps.querier
            .query_wasm_smart::<cw20::BalanceResponse>(
                config.credits_contract_address,
                &CreditsQueryMsg::BalanceAtHeight {
                    height: Some(height),
                    address,
                },
            )?
            .balance
    };

    Ok(VotingPowerAtHeightResponse {
        power: balance,
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

    let airdrop_balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.credits_contract_address.clone(),
        &CreditsQueryMsg::BalanceAtHeight {
            height: Some(height),
            address: config.airdrop_contract_address.to_string(),
        },
    )?;

    let total_supply: TotalSupplyResponse = deps.querier.query_wasm_smart(
        config.credits_contract_address,
        &CreditsQueryMsg::TotalSupplyAtHeight {
            height: Some(height),
        },
    )?;

    Ok(TotalPowerAtHeightResponse {
        power: total_supply
            .total_supply
            .checked_sub(airdrop_balance.balance)?,
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
    let config = CONFIG.load(deps.storage)?;
    to_binary(&config.name)
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
