use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, BLACKLISTED_ADDRESSES, CONFIG, DAO};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw2::set_contract_version;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use neutron_staking_tracker_common::msg::QueryMsg as TrackerQueryMsg;
use std::collections::HashSet;

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
    let vesting_contract_address = deps
        .api
        .addr_validate(&msg.staking_tracker_contract_address)?;

    let config = Config {
        staking_tracker_contract_address: vesting_contract_address,
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
        .add_attribute(
            "staking_tracker_contract_address",
            config.staking_tracker_contract_address,
        )
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
            staking_tracker_contract_address: vesting_contract_address,
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
        ExecuteMsg::AddToBlacklist { addresses } => {
            execute_add_to_blacklist(deps, env, info, addresses)
        }
        ExecuteMsg::RemoveFromBlacklist { addresses } => {
            execute_remove_from_blacklist(deps, env, info, addresses)
        }
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
    staking_tracker_contract_address: Option<String>,
    new_owner: Option<String>,
    new_description: Option<String>,
    new_name: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let staking_tracker_contract_address = staking_tracker_contract_address
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
    if let Some(new_vesting_contract_address) = staking_tracker_contract_address {
        config.staking_tracker_contract_address = new_vesting_contract_address;
    }

    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute(
            "staking_tracker_contract_address",
            config.staking_tracker_contract_address,
        )
        .add_attribute("owner", config.owner))
}

pub fn execute_add_to_blacklist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let validated_addresses: Vec<Addr> = addresses
        .iter()
        .map(|x| deps.api.addr_validate(&x))
        .collect::<StdResult<_>>()?;

    if let Some(mut blacklisted_addresses) = BLACKLISTED_ADDRESSES.may_load(deps.storage)? {
        let blacklisted_addresses_set: HashSet<Addr> =
            blacklisted_addresses.iter().cloned().collect();

        blacklisted_addresses.extend(
            validated_addresses
                .into_iter()
                .filter(|x| !blacklisted_addresses_set.contains(x)),
        );

        BLACKLISTED_ADDRESSES.save(deps.storage, &blacklisted_addresses, env.block.height)?;
    } else {
        BLACKLISTED_ADDRESSES.save(deps.storage, &validated_addresses, env.block.height)?;
    }

    Ok(Response::new()
        .add_attribute("action", "add_to_blacklist")
        .add_attribute("added_addresses", format!("{:?}", addresses)))
}

pub fn execute_remove_from_blacklist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(mut blacklisted_addresses) = BLACKLISTED_ADDRESSES.may_load(deps.storage)? {
        let validated_addresses: HashSet<Addr> = addresses
            .iter()
            .map(|x| deps.api.addr_validate(&x))
            .collect::<StdResult<Vec<_>>>()?
            .into_iter()
            .collect();

        blacklisted_addresses.retain(|x| !validated_addresses.contains(x));

        BLACKLISTED_ADDRESSES.save(deps.storage, &blacklisted_addresses, env.block.height)?;
    }

    Ok(Response::new()
        .add_attribute("action", "remove_from_blacklist")
        .add_attribute("removed_addresses", format!("{:?}", addresses)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VotingPowerAtHeight { address, height } => {
            to_json_binary(&query_voting_power_at_height(deps, env, address, height)?)
        }
        QueryMsg::TotalPowerAtHeight { height } => {
            to_json_binary(&query_total_power_at_height(deps, env, height)?)
        }
        QueryMsg::Info {} => query_info(deps),
        QueryMsg::Dao {} => query_dao(deps),
        QueryMsg::Name {} => query_name(deps),
        QueryMsg::Description {} => query_description(deps),
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::BondingStatus { height, address } => {
            to_json_binary(&query_bonding_status(deps, env, height, address)?)
        }
        QueryMsg::ListBonders { start_after, limit } => {
            query_list_bonders(deps, env, start_after, limit)
        }
        QueryMsg::ListBlacklistedAddresses { start_after, limit } => {
            to_json_binary(&query_list_blacklisted_addresses(deps, start_after, limit)?)
        }
        QueryMsg::IsAddressBlacklisted { address } => {
            to_json_binary(&query_is_address_blacklisted(deps, address)?)
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
    let addr = Addr::unchecked(address.clone());

    if let Some(blacklisted_addresses) =
        BLACKLISTED_ADDRESSES.may_load_at_height(deps.storage, height)?
    {
        if blacklisted_addresses.contains(&addr) {
            return Ok(VotingPowerAtHeightResponse {
                power: Uint128::zero(),
                height,
            });
        }
    }

    let config = CONFIG.load(deps.storage)?;

    let total_power: Uint128 = deps.querier.query_wasm_smart(
        config.staking_tracker_contract_address,
        &TrackerQueryMsg::StakeAtHeight {
            address,
            height: Some(height),
        },
    )?;

    Ok(VotingPowerAtHeightResponse {
        power: total_power,
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

    let total_power: Uint128 = deps.querier.query_wasm_smart(
        &config.staking_tracker_contract_address,
        &TrackerQueryMsg::TotalStakeAtHeight {
            height: Some(height),
        },
    )?;

    // sum voting power of blacklisted addresses
    let mut blacklisted_power = Uint128::zero();
    if let Some(blacklisted_addresses) =
        BLACKLISTED_ADDRESSES.may_load_at_height(deps.storage, height)?
    {
        for address in blacklisted_addresses {
            let power: Uint128 = deps.querier.query_wasm_smart(
                &config.staking_tracker_contract_address,
                &TrackerQueryMsg::StakeAtHeight {
                    address: address.to_string(),
                    height: Some(height),
                },
            )?;
            blacklisted_power = blacklisted_power.checked_add(power)?;
        }
    }

    // subtract blacklisted voting power
    let net_power = total_power.checked_sub(blacklisted_power)?;

    Ok(TotalPowerAtHeightResponse {
        power: net_power,
        height,
    })
}

pub fn query_info(deps: Deps) -> StdResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    to_json_binary(&cwd_interface::voting::InfoResponse { info })
}

pub fn query_dao(deps: Deps) -> StdResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    to_json_binary(&dao)
}

pub fn query_name(deps: Deps) -> StdResult<Binary> {
    let config: Config = CONFIG.load(deps.storage)?;
    to_json_binary(&config.name)
}

pub fn query_description(deps: Deps) -> StdResult<Binary> {
    let config: Config = CONFIG.load(deps.storage)?;
    to_json_binary(&config.description)
}

pub fn query_list_blacklisted_addresses(
    deps: Deps,
    start_after: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Vec<Addr>> {
    if let Some(mut blacklisted_addresses) = BLACKLISTED_ADDRESSES.may_load(deps.storage)? {
        let tail = match start_after {
            Some(start_after) => blacklisted_addresses.split_off(start_after.try_into().unwrap()),
            None => blacklisted_addresses,
        };

        return match limit {
            Some(limit) => Ok(tail.into_iter().take(limit.try_into().unwrap()).collect()),
            None => Ok(tail),
        };
    }

    Ok(vec![])
}

pub fn query_is_address_blacklisted(deps: Deps, address: String) -> StdResult<bool> {
    let addr = Addr::unchecked(address);

    match BLACKLISTED_ADDRESSES.may_load(deps.storage)? {
        Some(blacklisted_addresses) => Ok(blacklisted_addresses.contains(&addr)),
        None => Ok(false),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
