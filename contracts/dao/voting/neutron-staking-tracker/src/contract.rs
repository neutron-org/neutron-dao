use crate::state::{BONDED_VALIDATORS_SET, CONFIG, DELEGATIONS, VALIDATORS};
use neutron_staking_info_proxy_common::msg::ExecuteMsg as StakingInfoProxyExecuteMsg;
use neutron_staking_tracker_common::error::ContractError;
use neutron_staking_tracker_common::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg,
};
use neutron_staking_tracker_common::types::{Config, Delegation, Validator};
use std::collections::HashSet;
use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use neutron_std::types::cosmos::staking::v1beta1::{QueryValidatorResponse, StakingQuerier};
use std::str::FromStr;

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-staking-tracker";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REPLY_ON_AFTER_DELEGATION_MODIFIED_ERROR_STAKING_PROXY_ID: u64 = 1;
const REPLY_ON_BEFORE_VALIDATOR_SLASHED_ERROR_STAKING_PROXY_ID: u64 = 2;
const REPLY_ON_AFTER_VALIDATOR_BEGIN_UNBONDING_ERROR_STAKING_PROXY_ID: u64 = 3;
const REPLY_ON_AFTER_VALIDATOR_BONDED_ERROR_STAKING_PROXY_ID: u64 = 4;
const REPLY_ON_BEFORE_DELEGATION_REMOVED_ERROR_STAKING_PROXY_ID: u64 = 5;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    let staking_proxy_info_contract_address = msg
        .staking_proxy_info_contract_address
        .map(|s| deps.api.addr_validate(&s))
        .transpose()?;

    let config = Config {
        name: msg.name,
        description: msg.description,
        owner,
        staking_proxy_info_contract_address,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    BONDED_VALIDATORS_SET.save(deps.storage, &Vec::new(), env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("name", config.name)
        .add_attribute("description", config.description)
        .add_attribute("owner", config.owner)
        .add_attribute(
            "staking_proxy_info_contract_address",
            config
                .staking_proxy_info_contract_address
                .map(|a| a.to_string())
                .unwrap_or_default(),
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
            owner,
            name,
            description,
            staking_proxy_info_contract_address,
        } => execute_update_config(
            deps,
            info,
            owner,
            name,
            description,
            staking_proxy_info_contract_address,
        ),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    name: Option<String>,
    description: Option<String>,
    staking_proxy_info_contract_address: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }
    if let Some(name) = name {
        config.name = name;
    }
    if let Some(description) = description {
        config.description = description;
    }
    if let Some(staking_proxy_info_contract_address) = staking_proxy_info_contract_address {
        config.staking_proxy_info_contract_address = Some(
            deps.api
                .addr_validate(&staking_proxy_info_contract_address)?,
        );
    }
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute("owner", config.owner)
        .add_attribute(
            "staking_proxy_info_contract_address",
            config
                .staking_proxy_info_contract_address
                .map(|a| a.to_string())
                .unwrap_or_default(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::AfterValidatorCreated { val_addr } => after_validator_created(deps, env, val_addr),
        SudoMsg::AfterValidatorRemoved {
            cons_addr,
            val_addr,
        } => after_validator_removed(deps, env, cons_addr, val_addr),
        SudoMsg::AfterValidatorBonded {
            cons_addr,
            val_addr,
        } => after_validator_bonded(deps, env, cons_addr, val_addr),
        SudoMsg::AfterValidatorBeginUnbonding {
            cons_addr,
            val_addr,
        } => after_validator_begin_unbonding(deps, env, cons_addr, val_addr),
        SudoMsg::AfterDelegationModified { del_addr, val_addr } => {
            after_delegation_modified(deps, env, del_addr, val_addr)
        }
        SudoMsg::BeforeDelegationRemoved { del_addr, val_addr } => {
            before_delegation_removed(deps, env, del_addr, val_addr)
        }
        SudoMsg::BeforeValidatorSlashed { val_addr, fraction } => {
            before_validator_slashed(deps, env, val_addr, fraction)
        }
    }
}

pub(crate) fn after_validator_created(
    deps: DepsMut,
    env: Env,
    valoper_address: String,
) -> Result<Response, ContractError> {
    // Retrieve the consensus address using a helper function
    let valoper_addr = Addr::unchecked(&valoper_address);

    let querier = StakingQuerier::new(&deps.querier);
    let validator_data = querier
        .validator(valoper_address.clone())?
        .validator
        .ok_or_else(|| ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        })?;

    let total_tokens = Uint128::from_str(&validator_data.tokens)?;
    let total_shares = Uint128::from_str(&validator_data.delegator_shares)?;

    let new_validator = Validator {
        oper_address: valoper_addr.clone(),
        total_tokens,
        total_shares,
    };

    // Use `valoper_address` as the primary key for storage
    VALIDATORS.save(
        deps.storage,
        &valoper_addr, // Primary key is now `valoper`
        &new_validator,
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "after_validator_created")
        .add_attribute("operator_address", valoper_address)
        .add_attribute("total_tokens", total_tokens.to_string())
        .add_attribute("total_shares", total_shares.to_string()))
}

pub(crate) fn after_validator_removed(
    deps: DepsMut,
    env: Env,
    valoper_address: String,
    _valcons_address: String,
) -> Result<Response, ContractError> {
    let validator_addr = Addr::unchecked(&valoper_address);

    // Remove validator
    VALIDATORS.remove(deps.storage, &validator_addr, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "after_validator_removed")
        .add_attribute("valoper_address", valoper_address))
}

pub(crate) fn after_validator_bonded(
    deps: DepsMut,
    env: Env,
    _valcons_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let valoper_addr = Addr::unchecked(&valoper_address);

    let querier = StakingQuerier::new(&deps.querier);

    // Query the latest validator state from the chain
    let validator_data: QueryValidatorResponse = querier.validator(valoper_address.clone())?;

    let validator_info = validator_data
        .validator
        .ok_or(ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        })?;

    let total_tokens = Uint128::from_str(&validator_info.tokens)?;
    let total_shares = Uint128::from_str(&validator_info.delegator_shares)?;

    // Load validator or initialize if missing
    let mut validator = VALIDATORS
        .may_load(deps.storage, &valoper_addr)?
        .unwrap_or(Validator {
            oper_address: valoper_addr.clone(),
            total_tokens: Uint128::zero(),
            total_shares: Uint128::zero(),
        });

    // Update validator state
    validator.total_tokens = total_tokens;
    validator.total_shares = total_shares;
    // Save updated validator using valoper as the key
    VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;

    let mut resp = Response::new();

    let mut bonded_validators = BONDED_VALIDATORS_SET.load(deps.storage)?;
    // Defensive check
    if !bonded_validators.contains(&valoper_addr.to_string()) {
        bonded_validators.push(valoper_addr.to_string());
        BONDED_VALIDATORS_SET.save(deps.storage, &bonded_validators, env.block.height)?;

        // Call proxy info to notify about change of stake
        resp = with_slashing_event(
            resp,
            deps.as_ref(),
            REPLY_ON_AFTER_VALIDATOR_BONDED_ERROR_STAKING_PROXY_ID,
        )?;
    }

    Ok(resp
        .add_attribute("action", "after_validator_bonded")
        .add_attribute("valoper_address", valoper_address)
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string()))
}

pub fn before_validator_slashed(
    deps: DepsMut,
    env: Env,
    valoper_address: String,
    slashing_fraction: Decimal256,
) -> Result<Response, ContractError> {
    let mut resp = Response::new()
        .add_attribute("action", "before_validator_slashed")
        .add_attribute("valoper_address", &valoper_address)
        .add_attribute("slashing_fraction", slashing_fraction.to_string());

    let validator_addr = Addr::unchecked(&valoper_address);

    // Load validator state
    let validator = VALIDATORS.may_load(deps.storage, &validator_addr)?;

    // Defensive check
    if let Some(mut validator) = validator {
        // Calculate slashed tokens using Decimal256 multiplication and ceiling conversion
        let slashed_tokens: Uint256 = slashing_fraction
            .mul(Decimal256::from_atomics(validator.total_tokens, 0)?)
            .to_uint_ceil();

        let slashed_tokens_uint128: Uint128 = slashed_tokens.try_into()?;

        // Ensure tokens are reduced but not negative
        validator.total_tokens = validator.total_tokens.checked_sub(slashed_tokens_uint128)?;

        // Save updated validator state
        VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

        resp = with_slashing_event(
            resp,
            deps.as_ref(),
            REPLY_ON_BEFORE_VALIDATOR_SLASHED_ERROR_STAKING_PROXY_ID,
        )?;

        resp = resp
            .add_attribute("total_tokens", validator.total_tokens.to_string())
            .add_attribute("total_shares", validator.total_shares.to_string())
    }

    Ok(resp)
}

pub(crate) fn after_validator_begin_unbonding(
    deps: DepsMut,
    env: Env,
    _valcons_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let mut resp = Response::new()
        .add_attribute("action", "after_validator_begin_unbonding")
        .add_attribute("valoper_address", valoper_address.clone())
        .add_attribute("unbonding_start_height", env.block.height.to_string());

    let valoper_addr = Addr::unchecked(valoper_address);

    let mut bonded_vals = BONDED_VALIDATORS_SET.load(deps.storage)?;
    if !bonded_vals.contains(&valoper_addr.to_string()) {
        return Ok(resp);
    }

    // Mark validator as unbonded
    bonded_vals.retain(|a| a != &valoper_addr.to_string());
    BONDED_VALIDATORS_SET.save(deps.storage, &bonded_vals, env.block.height)?;

    // Call proxy info to notify about change of stake
    resp = with_slashing_event(
        resp,
        deps.as_ref(),
        REPLY_ON_AFTER_VALIDATOR_BEGIN_UNBONDING_ERROR_STAKING_PROXY_ID,
    )?;

    Ok(resp)
}

pub(crate) fn after_delegation_modified(
    deps: DepsMut,
    env: Env,
    delegator_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let delegator = deps.api.addr_validate(&delegator_address)?;
    let valoper_addr = Addr::unchecked(&valoper_address);

    let querier = StakingQuerier::new(&deps.querier);

    // Query **current delegation state** from the chain (fallback to zero if query fails)
    let current_shares = querier
        .delegation(delegator_address.clone(), valoper_address.clone())
        .ok() // If query fails, treat as no delegation
        .and_then(|delegation_info| {
            delegation_info
                .delegation_response
                .and_then(|resp| resp.delegation)
                .map(|del| {
                    Uint128::from_str(&del.shares).map_err(|e| ContractError::InvalidSharesFormat {
                        shares_str: del.shares.clone(),
                        err: e.to_string(),
                    })
                })
        })
        .transpose()?
        .unwrap_or(Uint128::zero()); // Default to zero if delegation does not exist

    // Load from delegations or zeros shares one if this delegation is new.
    let previous_shares = DELEGATIONS
        .may_load(deps.storage, (&delegator, &valoper_addr))?
        .map(|d| d.shares)
        .unwrap_or(Uint128::zero());

    // **Ensure delegation is correctly overwritten with actual shares**
    DELEGATIONS.save(
        deps.storage,
        (&delegator, &valoper_addr),
        &Delegation {
            delegator_address: delegator.clone(),
            validator_address: valoper_addr.clone(),
            shares: current_shares, // **Ensure correct shares are stored**
        },
        env.block.height,
    )?;

    // Load the validator by `valoper_address`.
    let mut validator = VALIDATORS.load(deps.storage, &valoper_addr)?;

    // https://github.com/neutron-org/cosmos-sdk/blob/83295e7c1380071cb9a0f405442d06acf387228c/x/staking/keeper/delegation.go#L1048
    // https://github.com/neutron-org/cosmos-sdk/blob/83295e7c1380071cb9a0f405442d06acf387228c/x/staking/keeper/validator.go#L148
    // - **Unbonding (Delegation Decrease)**:
    //   - When a delegator **removes** or **reduces** their delegation (`actual_shares < previous_shares`),
    //     we **cannot** rely on the chain's validator query because the state isn't updated yet due to the unbonding period.
    //   - Instead, we must **manually recalculate** the validator's `total_tokens` and `total_shares`
    //     by removing the undelegated shares.
    //
    // - **Bonding (Delegation Increase)**:
    //   - When a delegator **adds** or **increases** their delegation (`actual_shares >= previous_shares`),
    //     we can simply **query the validator's latest state** from the chain.
    //   - This is because the validator's `total_tokens` and `total_shares` are **already updated**
    //     by the time this function is executed.
    //
    if current_shares < previous_shares {
        let undelegated_shares = previous_shares - current_shares;

        validator.remove_del_shares(undelegated_shares)?;
    } else {
        // Query validator data to get the latest **total tokens & shares**
        let validator_data = querier
            .validator(valoper_address.clone())?
            .validator
            .ok_or_else(|| ContractError::ValidatorNotFound {
                address: valoper_address.clone(),
            })?;

        validator.total_shares = Uint128::from_str(&validator_data.delegator_shares)?;
        validator.total_tokens = Uint128::from_str(&validator_data.tokens)?;
    }

    // Save updated validator state
    VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;

    let resp = with_update_stake_msg(
        Response::new(),
        deps.as_ref(),
        &delegator,
        REPLY_ON_AFTER_DELEGATION_MODIFIED_ERROR_STAKING_PROXY_ID,
    )?;

    Ok(resp
        .add_attribute("action", "after_delegation_modified")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("valoper_address", valoper_address.to_string())
        .add_attribute("delegation_shares", current_shares.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string()))
}

pub(crate) fn before_delegation_removed(
    deps: DepsMut,
    env: Env,
    delegator_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let delegator = deps.api.addr_validate(&delegator_address)?;
    let valoper_addr = Addr::unchecked(valoper_address);

    let mut resp = Response::new();

    // Load shares amount we have for the delegation in the contract's state
    let shares = DELEGATIONS
        .may_load(deps.storage, (&delegator, &valoper_addr))?
        .map(|d| d.shares);

    // Defensive check
    if let Some(shares) = shares {
        // Load the validator by `valoper_address`.
        // The validator may not exist if they were unbonded when the delegation was removed.
        if let Some(mut validator) = VALIDATORS.may_load(deps.storage, &valoper_addr)? {
            // Since it's `before_delegation_removed`, we can safely remove all shares from validator
            validator.remove_del_shares(shares)?;

            // Save the updated validator state
            VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;
        }

        DELEGATIONS.remove(deps.storage, (&delegator, &valoper_addr), env.block.height)?;

        resp = with_update_stake_msg(
            resp,
            deps.as_ref(),
            &delegator,
            REPLY_ON_BEFORE_DELEGATION_REMOVED_ERROR_STAKING_PROXY_ID,
        )?;
    }

    Ok(resp
        .add_attribute("action", "before_delegation_removed")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("valoper_address", valoper_addr.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::StakeAtHeight { address, height } => {
            to_json_binary(&query_stake_at_height(deps, env, address, height)?)
        }
        QueryMsg::TotalStakeAtHeight { height } => {
            to_json_binary(&query_total_stake_at_height(deps, env, height)?)
        }
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
    }
}

/// Calculates the stake of a delegator at a specific block height.
///
/// Uses `Uint256` for intermediate calculations to avoid precision loss and overflow,
/// then converts the final result back to `Uint128`.
///
pub fn calculate_stake_at_height(deps: Deps, address: Addr, height: u64) -> StdResult<Uint128> {
    let mut stake = Uint256::zero(); // Use Uint256 to avoid overflow
    let bonded_vals = BONDED_VALIDATORS_SET
        .may_load_at_height(deps.storage, height)?
        .unwrap_or_default();

    for val_oper_address in bonded_vals {
        if let Some(validator) = VALIDATORS.may_load_at_height(
            deps.storage,
            &Addr::unchecked(val_oper_address),
            height,
        )? {
            if let Some(delegation) = DELEGATIONS.may_load_at_height(
                deps.storage,
                (&address, &validator.oper_address),
                height,
            )? {
                let shares_256 = Uint256::from(delegation.shares);
                let total_tokens_256 = Uint256::from(validator.total_tokens);
                let total_shares_256 = Uint256::from(validator.total_shares);

                let delegation_stake_256 = shares_256
                    .checked_mul(total_tokens_256)?
                    .checked_div(total_shares_256)?;

                stake = stake.checked_add(delegation_stake_256)?;
            }
        }
    }

    // Convert back to Uint128 safely
    let stake_128 = Uint128::try_from(stake)?;

    Ok(stake_128)
}

pub fn query_stake_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<Uint128> {
    let height = height.unwrap_or(env.block.height);
    let addr = deps.api.addr_validate(&address)?;
    let stake = calculate_stake_at_height(deps, addr, height)?;

    Ok(stake)
}

pub fn query_total_stake_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<Uint128> {
    let height = height.unwrap_or(env.block.height);

    let bonded_vals = BONDED_VALIDATORS_SET
        .may_load_at_height(deps.storage, height)?
        .unwrap_or_default()
        .into_iter()
        .collect::<HashSet<_>>();

    if bonded_vals.is_empty() {
        return Ok(Uint128::zero());
    }

    // calc total vp as usual
    let total_stake: Uint128 = bonded_vals
        .into_iter()
        .map(|valoper_addr| {
            VALIDATORS.may_load_at_height(deps.storage, &Addr::unchecked(valoper_addr), height)
        })
        .collect::<StdResult<Vec<Option<Validator>>>>()?
        .into_iter()
        .map(|m| m.map(|v| v.total_tokens).unwrap_or_default())
        .sum();

    Ok(total_stake)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if let Err(err) = msg.result.into_result() {
        Ok(Response::new()
            .add_attribute("reply_id", msg.id.to_string())
            .add_attribute("error", err))
    } else {
        Ok(Response::new().add_attribute("reply_id", msg.id.to_string()))
    }
}

fn with_update_stake_msg(
    resp: Response,
    deps: Deps,
    user: &Addr,
    reason: u64,
) -> Result<Response, ContractError> {
    // Call proxy info to notify about change of stake
    let config = CONFIG.load(deps.storage)?;
    if let Some(staking_proxy_info_contract_address) = config.staking_proxy_info_contract_address {
        let update_stake_msg = WasmMsg::Execute {
            contract_addr: staking_proxy_info_contract_address.to_string(),
            msg: to_json_binary(&StakingInfoProxyExecuteMsg::UpdateStake {
                user: user.to_string(),
            })?,
            funds: vec![],
        };

        // Use submsg because we want to ignore possible errors here.
        // This contract should be errorless no matter what.
        Ok(resp.add_submessage(SubMsg::reply_on_error(update_stake_msg, reason)))
    } else {
        Ok(resp)
    }
}

fn with_slashing_event(resp: Response, deps: Deps, reason: u64) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    Ok(
        if let Some(staking_proxy_info_contract_address) =
            config.staking_proxy_info_contract_address
        {
            let slashing_msg = WasmMsg::Execute {
                contract_addr: staking_proxy_info_contract_address.to_string(),
                msg: to_json_binary(&StakingInfoProxyExecuteMsg::Slashing {})?,
                funds: vec![],
            };

            // Use SubMsg because we want to ignore possible errors here.
            // This contract should be errorless no matter what.
            resp.add_submessage(SubMsg::reply_on_error(slashing_msg, reason))
        } else {
            resp
        },
    )
}
