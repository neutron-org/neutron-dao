use crate::error::ContractError;
use crate::error::ContractError::{NoStakingRewardsContractSet, Unauthorized};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, ProviderStakeQuery, ProvidersResponse,
    QueryMsg,
};
use crate::state::{Config, CONFIG, PROVIDERS};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use neutron_staking_rewards::msg::ExecuteMsg::{
    Slashing as RewardsMsgSlashing, UpdateStake as RewardsMsgUpdateStake,
};

const CONTRACT_NAME: &str = "crates.io:neutron-staking-info-proxy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    let staking_rewards: Option<Addr> = msg
        .staking_rewards
        .map(|s| deps.api.addr_validate(&s))
        .transpose()?;
    let config = Config {
        owner,
        staking_rewards,
        staking_denom: msg.staking_denom,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    for provider in msg.providers.iter() {
        let addr = deps.api.addr_validate(provider)?;
        PROVIDERS.save(deps.storage, addr, &())?;
    }

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
            staking_rewards,
            staking_denom,
        } => update_config(deps, env, info, owner, staking_rewards, staking_denom),
        ExecuteMsg::UpdateProviders { providers } => update_providers(deps, env, info, providers),
        ExecuteMsg::UpdateStake { user } => update_stake(deps, env, info, user),
        ExecuteMsg::Slashing {} => slashing(deps, env, info),
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
    staking_rewards: Option<String>,
    staking_denom: Option<String>,
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
    if let Some(staking_rewards) = staking_rewards {
        config.staking_rewards = Some(deps.api.addr_validate(&staking_rewards)?);
    }
    if let Some(staking_denom) = staking_denom {
        config.staking_denom = staking_denom;
    }
    // Validate updated config and save
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner.to_string())
        .add_attribute(
            "staking_rewards",
            config
                .staking_rewards
                .map(|s| s.to_string())
                .unwrap_or_default()
                .to_string(),
        )
        .add_attribute("staking_denom", config.staking_denom.to_string()))
}

/// Sets new set of providers that will proxy stake info to rewards contract.
/// Only the current owner can call this method.
fn update_providers(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    providers: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Ensure only the contract owner can update the configuration
    if info.sender != config.owner {
        return Err(Unauthorized {});
    }

    // Set new providers instead of old ones
    PROVIDERS.clear(deps.storage);
    for provider in providers.iter() {
        let addr = deps.api.addr_validate(provider)?;
        PROVIDERS.save(deps.storage, addr, &())?;
    }

    Ok(Response::new()
        .add_attribute("action", "update_providers")
        .add_attribute("owner", config.owner.to_string()))
}

/// Proxies update_stake query from provider to the `config.staking_rewards` contract.
/// Only allowed for contracts in `PROVIDERS` set.
fn update_stake(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    user: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if !PROVIDERS.has(deps.storage, info.sender) {
        return Err(Unauthorized {});
    }

    let msg = WasmMsg::Execute {
        contract_addr: config
            .staking_rewards
            .ok_or(NoStakingRewardsContractSet {})?
            .to_string(),
        msg: to_json_binary(&RewardsMsgUpdateStake {
            user: user.to_string(),
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "update_stake")
        .add_attribute("user", user))
}

/// Proxies slashing events from provider to the `config.staking_rewards` contract.
/// Only allowed for contracts in `PROVIDERS` set.
fn slashing(deps: DepsMut, _: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if !PROVIDERS.has(deps.storage, info.sender) {
        return Err(Unauthorized {});
    }

    let msg = WasmMsg::Execute {
        contract_addr: config
            .staking_rewards
            .ok_or(NoStakingRewardsContractSet {})?
            .to_string(),
        msg: to_json_binary(&RewardsMsgSlashing {})?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "slashing"))
}

// ----------------------------------------
//  Queries
// ----------------------------------------
#[entry_point]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<cosmwasm_std::Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&query_config(deps)?)?),
        QueryMsg::Providers {} => Ok(to_json_binary(&query_providers(deps)?)?),
        QueryMsg::UserStake { address, height } => {
            Ok(to_json_binary(&query_user_stake(deps, address, height)?)?)
        }
    }
}

/// Returns config.
fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        staking_rewards: config.staking_rewards.map(|s| s.to_string()),
    })
}

/// Returns providers.
fn query_providers(deps: Deps) -> StdResult<ProvidersResponse> {
    let providers: Vec<String> = PROVIDERS
        .keys(deps.storage, None, None, Order::Ascending)
        .flat_map(|k| k.map(|k| k.to_string()))
        .collect();
    Ok(ProvidersResponse { providers })
}

/// Returns sum of stake of each provider.
/// Returns Err if any of PROVIDER queries returned Err.
fn query_user_stake(deps: Deps, address: String, height: u64) -> Result<Coin, ContractError> {
    let user_addr = deps.api.addr_validate(&address)?;
    let config = CONFIG.load(deps.storage)?;
    let providers: Vec<Addr> = PROVIDERS
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<Result<_, _>>()?;
    let amount = providers
        .into_iter()
        .map(|provider| query_voting_power(deps, user_addr.clone(), &provider, height))
        .collect::<Result<Vec<_>, _>>()? // error caught here immediately
        .into_iter()
        .sum::<Uint128>();
    Ok(Coin {
        amount,
        denom: config.staking_denom,
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

// ----------------------------------------
//  Internal Logic
// ----------------------------------------

/// Queries the user’s voting power from the external provider,
/// ensuring that the returned denom matches this contract’s expected staking_denom.
fn query_voting_power(
    deps: Deps,
    address: Addr,
    provider: &Addr,
    height: u64,
) -> Result<Uint128, ContractError> {
    let user_stake: Uint128 = deps.querier.query_wasm_smart(
        provider,
        &ProviderStakeQuery::VotingPowerAtHeight {
            address,
            height: Some(height),
        },
    )?;
    Ok(user_stake)
}
