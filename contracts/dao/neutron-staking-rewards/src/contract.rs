use cosmwasm_std::{
    coin, entry_point, to_json_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::error::ContractError::{DaoStakeChangeNotTracked, InvalidStakeDenom, Unauthorized};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, RewardsResponse, StakeQuery,
    StateResponse,
};
use crate::state::{Config, State, UserInfo, CONFIG, STATE, USERS};

const CONTRACT_NAME: &str = "crates.io:neutron-staking-rewards";
const CONTRACT_VERSION: &str = "0.1.0";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Validate addresses from the instantiate message.
    let owner = deps.api.addr_validate(&msg.owner)?;
    let dao_address = deps.api.addr_validate(&msg.dao_address)?;
    let staking_info_proxy = deps.api.addr_validate(&msg.staking_info_proxy)?;

    // Create and validate the contract configuration.
    let config = Config {
        owner,
        dao_address,
        staking_info_proxy,
        annual_reward_rate_bps: msg.annual_reward_rate_bps,
        blocks_per_year: msg.blocks_per_year,
        staking_denom: msg.staking_denom,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    // Initialize the reward distribution state.
    // The global_reward_index tracks the global index for reward distribution
    // last_global_update_block records the block at which the global index was updated
    let state = State {
        global_reward_index: Decimal::zero(),
        last_global_update_block: env.block.height,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", config.owner.to_string())
        .add_attribute("dao_address", config.dao_address.to_string())
        .add_attribute("staking_info_proxy", config.staking_info_proxy.to_string())
        .add_attribute(
            "annual_reward_rate_bps",
            config.annual_reward_rate_bps.to_string(),
        )
        .add_attribute("blocks_per_year", config.blocks_per_year.to_string()))
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
            annual_reward_rate_bps,
            blocks_per_year,
            staking_info_proxy,
            staking_denom,
        } => update_config(
            deps,
            env,
            info,
            owner,
            annual_reward_rate_bps,
            blocks_per_year,
            staking_info_proxy,
            staking_denom,
        ),
        // Updates the stake information for a particular user
        ExecuteMsg::UpdateStake { user } => update_stake(deps, env, info, user),
        // Claims any accrued rewards for the caller
        ExecuteMsg::ClaimRewards {} => claim_rewards(deps, env, info),
    }
}

/// Updates configuration parameters for the contract.
/// Only the current owner can call this method.
fn update_config(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    annual_reward_rate_bps: Option<u64>,
    blocks_per_year: Option<u64>,
    staking_info_proxy: Option<String>,
    staking_denom: Option<String>,
) -> Result<Response, ContractError> {
    // Load the existing configuration
    let mut config = CONFIG.load(deps.storage)?;

    // Ensure only the contract owner can update the configuration
    if info.sender != config.owner {
        return Err(Unauthorized {});
    }

    // First, update the global index before changing any reward-related parameters
    // to ensure consistent reward distribution.
    update_global_index(deps.branch(), env, config.clone())?;

    // Update fields
    if let Some(new_owner) = owner {
        config.owner = deps.api.addr_validate(&new_owner)?;
    }
    if let Some(r) = annual_reward_rate_bps {
        config.annual_reward_rate_bps = r;
    }
    if let Some(bpy) = blocks_per_year {
        config.blocks_per_year = bpy;
    }
    if let Some(proxy) = staking_info_proxy {
        config.staking_info_proxy = deps.api.addr_validate(&proxy)?;
    }
    if let Some(denom) = staking_denom {
        config.staking_denom = denom;
    }

    // Validate updated config and save
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner.to_string())
        .add_attribute(
            "annual_reward_rate_bps",
            config.annual_reward_rate_bps.to_string(),
        )
        .add_attribute("blocks_per_year", config.blocks_per_year.to_string())
        .add_attribute("staking_info_proxy", config.staking_info_proxy.to_string())
        .add_attribute("staking_denom", config.staking_denom))
}

/// Called by the staking_info_proxy to update a user’s staked amount in this contract’s state.
/// This keeps track of user-level reward data (pending rewards, reward index).
fn update_stake(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.staking_info_proxy {
        return Err(Unauthorized {});
    }

    // This contract does not track DAO’s stake changes. If the DAO address is involved, revert.
    let user_addr = deps.api.addr_validate(&user)?;
    if user_addr == config.dao_address {
        return Err(DaoStakeChangeNotTracked {});
    }

    // Global index update
    let new_global_index = update_global_index(deps.branch(), env, config.clone())?;

    // Load the user’s current info, or create a default if not present
    let mut user_info = load_user_or_default(
        deps.as_ref(),
        user_addr.clone(),
        config.staking_denom.clone(),
    )?;

    // Calculate and accumulate any pending rewards
    let pending_rewards = get_user_pending_rewards(
        user_info.clone(),
        new_global_index,
        config.staking_denom.clone(),
    )?;
    user_info.pending_rewards = pending_rewards;
    user_info.user_reward_index = new_global_index;

    // Query the user’s new staked amount from the staking_info_proxy
    user_info.stake = safe_query_user_stake(
        deps.as_ref(),
        user_addr.clone(),
        config.staking_info_proxy,
        config.staking_denom.clone(),
    )?;
    USERS.save(deps.storage, &user_addr.clone(), &user_info)?;

    Ok(Response::new()
        .add_attribute("action", "update_stake")
        .add_attribute("user", user_addr.clone()))
}

/// Allows a user to claim any pending rewards accrued for their stake.
fn claim_rewards(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Update global index to the latest state
    let new_global_index = update_global_index(deps.branch(), env, config.clone())?;

    // Load the user’s current info
    let mut user_info = load_user_or_default(
        deps.as_ref(),
        info.sender.clone(),
        config.staking_denom.clone(),
    )?;

    // Calculate pending rewards to pay the user, then immediately set to 0
    let pending_rewards = get_user_pending_rewards(
        user_info.clone(),
        new_global_index,
        config.staking_denom.clone(),
    )?;
    user_info.pending_rewards = coin(0u128, config.staking_denom);

    user_info.user_reward_index = new_global_index;
    USERS.save(deps.storage, &info.sender, &user_info)?;

    let resp = Response::new();
    let resp = if !pending_rewards.amount.is_zero() {
        resp.add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![pending_rewards.clone()],
        })
    } else {
        resp
    };

    Ok(resp
        .add_attribute("action", "claim_rewards")
        .add_attribute("recipient", info.sender.to_string())
        .add_attribute("amount", pending_rewards.to_string()))
}

// ----------------------------------------
//  Queries
// ----------------------------------------
#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<cosmwasm_std::Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&query_config(deps)?)?),
        QueryMsg::State {} => Ok(to_json_binary(&query_state(deps)?)?),
        QueryMsg::Rewards { user } => Ok(to_json_binary(&query_rewards(deps, env, user)?)?),
    }
}

/// Returns only the config (no state fields).
fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        dao_address: config.dao_address.to_string(),
        staking_info_proxy: config.staking_info_proxy.to_string(),
        annual_reward_rate_bps: config.annual_reward_rate_bps,
        blocks_per_year: config.blocks_per_year,
        staking_denom: config.staking_denom,
    })
}

/// Returns only the state info (global index and last update).
fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        global_reward_index: state.global_reward_index.to_string(),
        last_global_update_block: state.last_global_update_block,
    })
}

/// Returns how many rewards the user currently has pending, simulating a global index update at
/// the current block.
fn query_rewards(deps: Deps, env: Env, user: String) -> Result<RewardsResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let user_addr = deps.api.addr_validate(&user)?;

    // simulate a global index update
    let new_global_index = get_updated_global_index(
        &env,
        config.clone(),
        state.global_reward_index,
        state.last_global_update_block,
    )?;

    // simulate a user update
    // User update
    let user_info = load_user_or_default(deps, user_addr.clone(), config.staking_denom.clone())?;
    let pending_rewards = get_user_pending_rewards(
        user_info.clone(),
        new_global_index,
        config.staking_denom.clone(),
    )?;

    Ok(RewardsResponse { pending_rewards })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

// ----------------------------------------
//  Internal Logic
// ----------------------------------------

/// Updates the global reward index in state based on how many blocks have passed since last update.
fn update_global_index(deps: DepsMut, env: Env, config: Config) -> Result<Decimal, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    // Do a Global index update so that any new config changes apply afterward
    let new_global_index = get_updated_global_index(
        &env,
        config.clone(),
        state.global_reward_index,
        state.last_global_update_block,
    )?;
    state.global_reward_index = new_global_index;
    state.last_global_update_block = env.block.height;
    STATE.save(deps.storage, &state)?;

    Ok(new_global_index)
}

/// Computes what the global reward index should be, given the elapsed time and the configured
/// reward rate.
fn get_updated_global_index(
    env: &Env,
    config: Config,
    old_global_index: Decimal,
    last_global_update_block: u64,
) -> Result<Decimal, ContractError> {
    let current_block = env.block.height;
    if current_block <= last_global_update_block {
        return Ok(old_global_index);
    }

    // Calculate number of blocks since last global index update
    let delta_t = current_block - last_global_update_block;

    // Convert annual reward rate in basis points (bps) to a Decimal (e.g., 500 bps = 5%)
    let annual_rate = Decimal::from_ratio(config.annual_reward_rate_bps, 10_000u64);
    // Convert blocks_per_year to a Decimal
    let blocks_per_year = Decimal::from_atomics(config.blocks_per_year, 0).unwrap_or_default();
    // Reward rate per block = (annual_rate / blocks_per_year)
    let rate_per_block = annual_rate / blocks_per_year;

    // Increase in index over the time delta
    let delta_index = rate_per_block * Decimal::from_atomics(delta_t, 0).unwrap();

    // The new global index is the old index plus any delta over the elapsed blocks
    Ok(delta_index + old_global_index)
}

/// Loads user info from state, or returns a default if the user has no entry yet.
fn load_user_or_default(
    deps: Deps,
    user_addr: Addr,
    staking_denom: String,
) -> Result<UserInfo, ContractError> {
    let user_info = USERS
        .may_load(deps.storage, &user_addr)?
        .unwrap_or_else(|| UserInfo {
            user_reward_index: Decimal::zero(),
            stake: coin(0u128, staking_denom.clone()),
            pending_rewards: coin(0u128, staking_denom.clone()),
        });

    Ok(user_info)
}

/// Calculates a user’s pending rewards given their current stake, the global reward index, and
/// the user’s last recorded reward index.
fn get_user_pending_rewards(
    user_info: UserInfo,
    global_reward_index: Decimal,
    staking_denom: String,
) -> Result<Coin, ContractError> {
    let delta_index = global_reward_index - user_info.user_reward_index;
    if !delta_index.is_zero() && !user_info.stake.amount.is_zero() {
        let newly_accrued = coin(
            user_info.stake.amount.mul_floor(delta_index).u128(),
            staking_denom.clone(),
        );
        return Ok(coin(
            (user_info.pending_rewards.amount + newly_accrued.amount).u128(),
            staking_denom,
        ));
    }

    return Ok(user_info.pending_rewards);
}

/// Safely queries the user’s staked amount from the external staking_info_proxy,
/// ensuring that the returned denom matches this contract’s expected staking_denom.
fn safe_query_user_stake(
    deps: Deps,
    user_addr: Addr,
    staking_info_proxy: Addr,
    staking_denom: String,
) -> Result<Coin, ContractError> {
    let user_stake: Coin = deps.querier.query_wasm_smart(
        staking_info_proxy,
        &StakeQuery::User {
            address: user_addr.to_string(),
        },
    )?;
    if user_stake.denom != staking_denom {
        return Err(InvalidStakeDenom {});
    }

    return Ok(user_stake);
}
