use cosmwasm_std::{
    coin, entry_point, to_json_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::state::{
    assert_pause, is_allowed_to_pause, is_allowed_to_unpause, CONFIG, PAUSED, STATE, USERS,
};
use neutron_staking_info_proxy_common::msg::QueryMsg as InfoProxyQueryMsg;
use neutron_staking_rewards_common::error::ContractError;
use neutron_staking_rewards_common::error::ContractError::{
    ContractPaused, DaoStakeChangeNotTracked, InvalidStakeDenom, Unauthorized,
};
use neutron_staking_rewards_common::msg::{
    ConfigResponse, InstantiateMsg, MigrateMsg, QueryMsg, RewardsResponse, StateResponse,
};
use neutron_staking_rewards_common::msg::{ExecuteMsg, SlashingEventsResponse};
use neutron_staking_rewards_common::types::{Config, State, UserInfo};

const CONTRACT_NAME: &str = "crates.io:neutron-staking-rewards";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
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
    let security_address = deps.api.addr_validate(&msg.security_address)?;

    // Create and validate the contract configuration.
    let config = Config {
        owner,
        dao_address,
        staking_info_proxy,
        annual_reward_rate_bps: msg.annual_reward_rate_bps,
        blocks_per_year: msg.blocks_per_year,
        staking_denom: msg.staking_denom,
        security_address,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    // Initialize the reward distribution state.
    // The global_reward_index tracks the global index for reward distribution
    // last_global_update_block records the block at which the global index was updated
    let state = State {
        global_reward_index: Decimal::zero(),
        global_update_height: env.block.height,
        slashing_events: vec![],
    };
    STATE.save(deps.storage, &state)?;

    PAUSED.save(deps.storage, &false)?;

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
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
            security_address,
        } => update_config(
            deps,
            env,
            info,
            owner,
            annual_reward_rate_bps,
            blocks_per_year,
            staking_info_proxy,
            staking_denom,
            security_address,
        ),
        // Updates the stake information for a particular user
        ExecuteMsg::UpdateStake { user } => {
            // try to update stake, if an error happens during the update we need to pause the contract
            // (because an error is terrible and it breaks out internal accounting)
            // We don't need to pause the contract in cases:
            // * if it's Unauthorized error;
            // * if it's DaoStakeChangeNotTracked error;
            // * of if the contract is already paused.
            update_stake(deps.branch(), env, info, user).or_else(|err| match err {
                Unauthorized {} => Err(err),
                DaoStakeChangeNotTracked {} => Err(err),
                ContractPaused {} => Err(err),
                _ => {
                    PAUSED.save(deps.storage, &true)?;

                    Ok(Response::new().add_attribute("update_stake_error", format!("{}", err)))
                }
            })
        }
        // Updates the stake information for a particular user
        // try to account slashing, if an error happens during the method execution, we need to pause the contract
        // (because an error is terrible and it breaks out internal accounting)
        // We don't need to pause the contract in cases:
        // * if it's Unauthorized error;
        // * if it's DaoStakeChangeNotTracked error;
        // * of if the contract is already paused.
        ExecuteMsg::Slashing {} => slashing(deps.branch(), env, info).or_else(|err| match err {
            Unauthorized {} => Err(err),
            DaoStakeChangeNotTracked {} => Err(err),
            ContractPaused {} => Err(err),
            _ => {
                PAUSED.save(deps.storage, &true)?;

                Ok(Response::new().add_attribute("slashing_error", format!("{}", err)))
            }
        }),
        // Claims any accrued rewards for the caller
        ExecuteMsg::ClaimRewards { to_address } => claim_rewards(deps, env, info, to_address),
        // Pauses the contract
        ExecuteMsg::Pause {} => pause(deps, info),
        // Unpauses the contract
        ExecuteMsg::Unpause {} => unpause(deps, info),
    }
}

fn unpause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !is_allowed_to_unpause(&config, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    PAUSED.save(deps.storage, &false)?;
    Ok(Response::new())
}

fn pause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !is_allowed_to_pause(&config, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    PAUSED.save(deps.storage, &true)?;
    Ok(Response::new())
}

/// Updates configuration parameters for the contract.
/// Only the current owner can call this method.
#[allow(clippy::too_many_arguments)]
fn update_config(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    annual_reward_rate_bps: Option<u64>,
    blocks_per_year: Option<u64>,
    staking_info_proxy: Option<String>,
    staking_denom: Option<String>,
    security_address: Option<String>,
) -> Result<Response, ContractError> {
    // Load the existing configuration
    let mut config = CONFIG.load(deps.storage)?;

    // Ensure only the contract owner can update the configuration
    if info.sender != config.owner {
        return Err(Unauthorized {});
    }

    // First, update the global index before changing any reward-related parameters
    // to ensure consistent reward distribution.
    update_global_index(deps.branch(), &env, config.clone())?;

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
    if let Some(security_address) = security_address {
        config.security_address = deps.api.addr_validate(&security_address)?;
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
        .add_attribute("staking_denom", config.staking_denom)
        .add_attribute("security_address", config.security_address.to_string()))
}

/// Called by the staking_info_proxy to update a user’s staked amount in this contract’s state.
/// This keeps track of user-level reward data (pending rewards, reward index).
fn update_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user: String,
) -> Result<Response, ContractError> {
    // stake update is forbidden while the contract is on pause
    assert_pause(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.staking_info_proxy {
        return Err(Unauthorized {});
    }

    // This contract does not track DAO’s stake changes. If the DAO address is involved, revert.
    let user_addr = deps.api.addr_validate(&user)?;
    if user_addr == config.dao_address {
        return Err(DaoStakeChangeNotTracked {});
    }

    let (user_info, state) =
        process_slashing_events(deps.as_ref(), config.clone(), user_addr.clone())?;

    let updated_state = get_updated_state(&config, &state, env.block.height)?;
    let mut updated_user_info = get_updated_user_info(
        user_info,
        updated_state.global_reward_index,
        env.block.height,
        config.staking_denom.clone(),
    )?;
    // Set the user stake to current value
    updated_user_info.stake = safe_query_user_stake(
        &deps.as_ref(),
        user_addr.clone(),
        config.staking_info_proxy.clone(),
        config.staking_denom.clone(),
        env.block.height,
    )?;
    STATE.save(deps.storage, &updated_state)?;
    USERS.save(deps.storage, &user_addr.clone(), &updated_user_info)?;

    Ok(Response::new()
        .add_attribute("action", "update_stake")
        .add_attribute("user", user_addr.clone()))
}

/// Called by the staking_info_proxy when a slashing event happens. The staking_info_proxy should
/// only send a single Slashing message for a single height, i.e., if multiple validators were
/// slashed on height X, only one Slashing message must be sent.
fn slashing(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    assert_pause(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.staking_info_proxy {
        return Err(Unauthorized {});
    }

    let state = STATE.load(deps.storage)?;
    if let Some((_, last_event_height)) = state.slashing_events.last() {
        if *last_event_height >= env.block.height {
            return Ok(Response::new()
                .add_attribute("action", "slashing")
                .add_attribute("result", "ignored"));
        }
    }

    let mut updated_state = get_updated_state(&config, &state, env.block.height)?;
    updated_state.slashing_events.push((
        updated_state.global_reward_index,
        updated_state.global_update_height,
    ));
    STATE.save(deps.storage, &updated_state)?;

    Ok(Response::new()
        .add_attribute("action", "slashing")
        .add_attribute("result", "acknowledged")
        .add_attribute("block_height", format!("{}", env.block.height)))
}

/// Allows a user to claim any pending rewards accrued for their stake.
fn claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_address: Option<String>,
) -> Result<Response, ContractError> {
    // users can't claim rewards while the contract is on pause
    assert_pause(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    if info.sender == config.dao_address {
        return Err(Unauthorized {});
    }

    let (user_info, state) =
        process_slashing_events(deps.as_ref(), config.clone(), info.sender.clone())?;
    let updated_state = get_updated_state(&config, &state, env.block.height)?;
    let mut updated_user_info = get_updated_user_info(
        user_info,
        updated_state.global_reward_index,
        env.block.height,
        config.staking_denom.clone(),
    )?;
    let pending_rewards = updated_user_info.pending_rewards;
    updated_user_info.pending_rewards = coin(0u128, config.staking_denom);
    STATE.save(deps.storage, &updated_state)?;
    USERS.save(deps.storage, &info.sender, &updated_user_info)?;

    let recipient = to_address.unwrap_or(info.sender.to_string());
    let resp = Response::new();
    let resp = if !pending_rewards.amount.is_zero() {
        resp.add_message(BankMsg::Send {
            to_address: recipient.clone(),
            amount: vec![pending_rewards.clone()],
        })
    } else {
        resp
    };

    Ok(resp
        .add_attribute("action", "claim_rewards")
        .add_attribute("recipient", recipient)
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
        QueryMsg::SlashingEvents { from_height } => {
            Ok(to_json_binary(&query_slashing_events(deps, from_height)?)?)
        }
        QueryMsg::IsPaused {} => Ok(to_json_binary(&query_is_paused(deps)?)?),
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
        last_global_update_block: state.global_update_height,
    })
}

/// Returns slice of slashing events.
fn query_slashing_events(deps: Deps, from_height: u64) -> StdResult<SlashingEventsResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(SlashingEventsResponse {
        slashing_events: state.load_unprocessed_slashing_events(from_height),
    })
}

/// Returns true if contract is paused, false if not
fn query_is_paused(deps: Deps) -> StdResult<bool> {
    let is_paused = PAUSED.load(deps.storage)?;
    Ok(is_paused)
}

/// Returns how many rewards the user currently has pending, simulating a global index update at
/// the current block.
fn query_rewards(deps: Deps, env: Env, user: String) -> Result<RewardsResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let user_addr = deps.api.addr_validate(&user)?;

    let (user_info, state) = process_slashing_events(deps, config.clone(), user_addr)?;

    let updated_state = get_updated_state(&config, &state, env.block.height)?;
    let updated_user_info = get_updated_user_info(
        user_info,
        updated_state.global_reward_index,
        env.block.height,
        config.staking_denom.clone(),
    )?;

    Ok(RewardsResponse {
        pending_rewards: updated_user_info.pending_rewards,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

// ----------------------------------------------------------------------------
//  Internal Logic
// ----------------------------------------------------------------------------

fn get_updated_state(
    config: &Config,
    state: &State,
    new_height: u64,
) -> Result<State, ContractError> {
    let mut state = state.clone();

    let new_global_index = get_updated_global_index(
        config.clone(),
        new_height,
        state.global_reward_index,
        state.global_update_height,
    )?;
    state.global_reward_index = new_global_index;
    state.global_update_height = new_height;

    Ok(state)
}

fn get_updated_user_info(
    user_info: UserInfo,
    global_index: Decimal,
    new_height: u64,
    staking_denom: String,
) -> Result<UserInfo, ContractError> {
    let mut user_info = user_info.clone();

    // Calculate and accumulate any pending rewards
    let pending_rewards = get_user_pending_rewards(user_info.clone(), global_index, staking_denom)?;
    user_info.pending_rewards = pending_rewards;
    user_info.user_reward_index = global_index;
    user_info.last_update_block = new_height;

    Ok(user_info)
}

fn process_slashing_events(
    deps: Deps,
    config: Config,
    user_addr: Addr,
) -> Result<(UserInfo, State), ContractError> {
    let state = STATE.load(deps.storage)?;
    // Load the user’s current info, or create a default if not present
    let (mut user_info, existed) =
        load_user_or_default(deps, user_addr.clone(), config.staking_denom.clone())?;

    if existed {
        let slashing_events = state.load_unprocessed_slashing_events(user_info.last_update_block);
        for (slashing_event_global_index, slashing_event_height) in slashing_events.into_iter() {
            user_info = get_updated_user_info(
                user_info.clone(),
                slashing_event_global_index,
                slashing_event_height,
                config.staking_denom.clone(),
            )?;

            // Set the user stake to the value after the slashing event
            user_info.stake = safe_query_user_stake(
                &deps,
                user_addr.clone(),
                config.staking_info_proxy.clone(),
                config.staking_denom.clone(),
                slashing_event_height,
            )?;
        }
    }

    Ok((user_info, state))
}

/// Updates the global reward index in state based on how many blocks have passed since last update.
fn update_global_index(deps: DepsMut, env: &Env, config: Config) -> Result<Decimal, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    let new_global_index = get_updated_global_index(
        config.clone(),
        env.block.height,
        state.global_reward_index,
        state.global_update_height,
    )?;
    state.global_reward_index = new_global_index;
    state.global_update_height = env.block.height;
    STATE.save(deps.storage, &state)?;

    Ok(new_global_index)
}

/// Computes what the global reward index should be, given the elapsed time and the configured
/// reward rate.
fn get_updated_global_index(
    config: Config,
    current_block: u64,
    old_global_index: Decimal,
    last_global_update_block: u64,
) -> Result<Decimal, ContractError> {
    if current_block < last_global_update_block {
        return Err(ContractError::TriedGetGlobalIndexInThePast {
            current_block,
            last_global_update_block,
        });
    }

    if current_block == last_global_update_block {
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
/// Returns (UserInfo, DidUserExists) value.
fn load_user_or_default(
    deps: Deps,
    user_addr: Addr,
    staking_denom: String,
) -> Result<(UserInfo, bool), ContractError> {
    let maybe_user_info = USERS.may_load(deps.storage, &user_addr)?;
    let existed = maybe_user_info.is_some();
    let user_info = maybe_user_info.unwrap_or_else(|| UserInfo {
        user_reward_index: Decimal::zero(),
        stake: coin(0u128, staking_denom.clone()),
        last_update_block: 0u64,
        pending_rewards: coin(0u128, staking_denom.clone()),
    });
    Ok((user_info, existed))
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
            user_info
                .stake
                .amount
                .checked_mul_floor(delta_index)?
                .u128(),
            staking_denom.clone(),
        );
        return Ok(coin(
            (user_info.pending_rewards.amount + newly_accrued.amount).u128(),
            staking_denom,
        ));
    }

    Ok(user_info.pending_rewards)
}

/// Safely queries the user’s staked amount from the external staking_info_proxy,
/// ensuring that the returned denom matches this contract’s expected staking_denom.
fn safe_query_user_stake(
    deps: &Deps,
    user_addr: Addr,
    staking_info_proxy: Addr,
    staking_denom: String,
    height: u64,
) -> Result<Coin, ContractError> {
    let res: StdResult<Coin> = deps.querier.query_wasm_smart(
        staking_info_proxy,
        &InfoProxyQueryMsg::UserStake {
            address: user_addr.to_string(),
            // increment height because staking_tracker contract returns (n-1) data on
            // query_voting_power_at_height(n) and query_total_power_at_height(n)
            height: height + 1,
        },
    );

    match res {
        Err(err) => {
            let err_str = err.to_string();
            Err(ContractError::Std(StdError::generic_err(err_str)))
        }
        Ok(user_stake) => {
            if user_stake.denom != staking_denom {
                return Err(InvalidStakeDenom {
                    denom: user_stake.denom,
                });
            }

            Ok(user_stake)
        }
    }
}
