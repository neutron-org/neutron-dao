use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, ProxyInfoExecute, QueryMsg, SudoMsg};
use crate::state::{
    Config, Delegation, Validator, BLACKLISTED_ADDRESSES, BONDED_VALIDATORS, CONFIG, DAO,
    DELEGATIONS, VALIDATORS,
};
use std::collections::HashSet;
use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Order, Reply,
    Response, StdError, StdResult, SubMsg, Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use neutron_std::types::cosmos::staking::v1beta1::{QueryValidatorResponse, StakingQuerier};
use std::str::FromStr;

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-staking-tracker";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REPLY_ON_AFTER_DELEGATION_MODIFIED_ERROR_STAKING_PROXY_ID: u64 = 1;
const REPLY_ON_BEFORE_VALIDATOR_SLASHED_ERROR_STAKING_PROXY_ID: u64 = 2;
const REPLY_ON_AFTER_VALIDATOR_BEGIN_UNBONDING_ERROR_STAKING_PROXY_ID: u64 = 3;
const REPLY_ON_AFTER_VALIDATOR_BONDED_ERROR_STAKING_PROXY_ID: u64 = 4;
const REPLY_ON_ADD_TO_BLACKLIST_ERROR_STAKING_PROXY_ID: u64 = 5;
const REPLY_ON_REMOVE_FROM_BLACKLIST_ERROR_STAKING_PROXY_ID: u64 = 6;
const REPLY_ON_BEFORE_DELEGATION_REMOVED_ERROR_STAKING_PROXY_ID: u64 = 7;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
    DAO.save(deps.storage, &info.sender)?;
    BONDED_VALIDATORS.save(deps.storage, &Vec::new(), env.block.height)?;

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
        ExecuteMsg::AddToBlacklist { addresses } => execute_add_to_blacklist(deps, info, addresses),
        ExecuteMsg::RemoveFromBlacklist { addresses } => {
            execute_remove_from_blacklist(deps, info, addresses)
        }
    }
}

pub fn execute_add_to_blacklist(
    deps: DepsMut,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut resp = Response::new();
    for address in &addresses {
        let addr = deps.api.addr_validate(address)?;
        resp = with_update_stake_msg(
            resp,
            deps.as_ref(),
            &addr,
            REPLY_ON_ADD_TO_BLACKLIST_ERROR_STAKING_PROXY_ID,
        )?;
        BLACKLISTED_ADDRESSES.save(deps.storage, addr, &true)?;
    }

    Ok(resp
        .add_attribute("action", "add_to_blacklist")
        .add_attribute("added_addresses", format!("{:?}", addresses)))
}

pub fn execute_remove_from_blacklist(
    deps: DepsMut,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut resp = Response::new();
    for address in &addresses {
        let addr = deps.api.addr_validate(address)?;
        resp = with_update_stake_msg(
            resp,
            deps.as_ref(),
            &addr,
            REPLY_ON_REMOVE_FROM_BLACKLIST_ERROR_STAKING_PROXY_ID,
        )?;
        BLACKLISTED_ADDRESSES.remove(deps.storage, addr);
    }

    Ok(resp
        .add_attribute("action", "remove_from_blacklist")
        .add_attribute("removed_addresses", format!("{:?}", addresses)))
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
            active: true,
        });

    // Update validator state
    validator.total_tokens = total_tokens;
    validator.total_shares = total_shares;
    // Save updated validator using valoper as the key
    VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;

    let mut resp = Response::new();

    let mut bonded_validators = BONDED_VALIDATORS.load(deps.storage)?;
    if !bonded_validators.contains(&valoper_addr.to_string()) {
        bonded_validators.push(valoper_addr.to_string());
        BONDED_VALIDATORS.save(deps.storage, &bonded_validators, env.block.height)?;

        // Call proxy info to notify about change of stake
        resp = with_slashing_event(
            resp,
            deps.as_ref(),
            REPLY_ON_AFTER_VALIDATOR_BONDED_ERROR_STAKING_PROXY_ID,
        )?;
    }

    Ok(resp
        .add_attribute("action", "validator_bonded")
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
    let validator_addr = Addr::unchecked(&valoper_address);

    // Load validator state
    let mut validator = VALIDATORS.load(deps.storage, &validator_addr)?;

    // Calculate slashed tokens using Decimal256 multiplication and ceiling conversion
    let slashed_tokens: Uint256 = slashing_fraction
        .mul(Decimal256::from_atomics(validator.total_tokens, 0)?)
        .to_uint_ceil();

    let slashed_tokens_uint128: Uint128 = slashed_tokens.try_into()?;

    // Ensure tokens are reduced but not negative
    validator.total_tokens = validator.total_tokens.checked_sub(slashed_tokens_uint128)?;

    // Save updated validator state
    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    let resp = with_slashing_event(
        Response::new(),
        deps.as_ref(),
        REPLY_ON_BEFORE_VALIDATOR_SLASHED_ERROR_STAKING_PROXY_ID,
    )?;

    Ok(resp
        .add_attribute("action", "before_validator_slashed")
        .add_attribute("valoper_address", valoper_address)
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("slashing_fraction", slashing_fraction.to_string()))
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

    let mut bonded_vals = BONDED_VALIDATORS.load(deps.storage)?;
    if !bonded_vals.contains(&valoper_addr.to_string()) {
        return Ok(resp);
    }

    // Mark validator as unbonded
    bonded_vals.retain(|a| a != &valoper_addr.to_string());
    BONDED_VALIDATORS.save(deps.storage, &bonded_vals, env.block.height)?;

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
    let actual_shares = querier
        .delegation(delegator_address.clone(), valoper_address.clone())
        .ok() // If query fails, treat as no delegation
        .and_then(|delegation_info| {
            delegation_info
                .delegation_response
                .and_then(|resp| resp.delegation)
                .map(|del| {
                    Uint128::from_str(&del.shares).map_err(|_| ContractError::InvalidSharesFormat {
                        shares_str: del.shares.clone(),
                    })
                })
        })
        .transpose()?
        .unwrap_or(Uint128::zero()); // Default to zero if delegation does not exist

    let previous_shares = DELEGATIONS
        .may_load(deps.storage, (&delegator, &valoper_addr))?
        .unwrap_or(Delegation {
            delegator_address: delegator.clone(),
            validator_address: valoper_addr.clone(),
            shares: Uint128::zero(),
        })
        .shares;

    // **Ensure delegation is correctly overwritten with actual shares**
    DELEGATIONS.save(
        deps.storage,
        (&delegator, &valoper_addr),
        &Delegation {
            delegator_address: delegator.clone(),
            validator_address: valoper_addr.clone(),
            shares: actual_shares, // **Ensure correct shares are stored**
        },
        env.block.height,
    )?;

    let mut resp = Response::new()
        .add_attribute("action", "after_delegation_modified")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("valoper_address", valoper_address.to_string())
        .add_attribute("delegation_shares", actual_shares.to_string());

    // Load validator by `valoper_address`.
    // If validator not found, that means he's not bonded. This means we shouldn't track his state.
    if let Some(mut validator) = VALIDATORS.may_load(deps.storage, &valoper_addr)? {
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
        if actual_shares < previous_shares {
            let undelegated_shares = previous_shares - actual_shares;

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

        resp = resp
            .add_attribute("total_shares", validator.total_shares.to_string())
            .add_attribute("total_tokens", validator.total_tokens.to_string());

        // Do not need to trigger rewards recalculation if delegation was to unbonded validator,
        // since it does not change the stake.
        resp = with_update_stake_msg(
            resp,
            deps.as_ref(),
            &delegator,
            REPLY_ON_AFTER_DELEGATION_MODIFIED_ERROR_STAKING_PROXY_ID,
        )?;
    }

    Ok(resp)
}

pub(crate) fn before_delegation_removed(
    deps: DepsMut,
    env: Env,
    delegator_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let delegator = deps.api.addr_validate(&delegator_address)?;
    let valoper_addr = Addr::unchecked(valoper_address);

    // Load shares amount we have for the delegation in the contract's state
    let mut delegation = DELEGATIONS
        .may_load(deps.storage, (&delegator, &valoper_addr))?
        .unwrap_or(Delegation {
            delegator_address: delegator.clone(),
            validator_address: valoper_addr.clone(),
            shares: Uint128::zero(),
        });

    let previous_shares = delegation.shares;

    // Load validator by `valoper_address`.
    // Validator can not exist if it was unbonded when delegation was removed.
    if let Some(mut validator) = VALIDATORS.may_load(deps.storage, &valoper_addr)? {
        // Since it's `before_delegation_removed`, we can safely remove all shares from validator
        validator.remove_del_shares(previous_shares)?;

        // Save the updated validator state
        VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;
    }

    delegation.shares = Uint128::zero();
    DELEGATIONS.save(
        deps.storage,
        (&delegator, &valoper_addr),
        &delegation,
        env.block.height,
    )?;

    let resp = with_update_stake_msg(
        Response::new(),
        deps.as_ref(),
        &delegator,
        REPLY_ON_BEFORE_DELEGATION_REMOVED_ERROR_STAKING_PROXY_ID,
    )?;

    Ok(resp
        .add_attribute("action", "before_delegation_removed")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("valoper_address", valoper_addr.to_string()))
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
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::ListBlacklistedAddresses { start_after, limit } => {
            to_json_binary(&query_list_blacklisted_addresses(deps, start_after, limit)?)
        }
        QueryMsg::IsAddressBlacklisted { address } => {
            to_json_binary(&query_is_address_blacklisted(deps, address)?)
        }
    }
}

pub fn query_list_blacklisted_addresses(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<Addr>> {
    let start = start_after.map(Bound::exclusive); // Convert to exclusive Bound

    let limit = limit.unwrap_or(10) as usize;

    let blacklisted: Vec<Addr> = BLACKLISTED_ADDRESSES
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(blacklisted)
}

pub fn query_is_address_blacklisted(deps: Deps, address: String) -> StdResult<bool> {
    let addr = Addr::unchecked(address);
    let is_blacklisted = BLACKLISTED_ADDRESSES
        .may_load(deps.storage, addr)?
        .unwrap_or(false);
    Ok(is_blacklisted)
}

/// Calculates the voting power of a delegator at a specific block height.
///
/// Uses `Uint256` for intermediate calculations to avoid precision loss and overflow,
/// then converts the final result back to `Uint128`.
///
pub fn calculate_voting_power(deps: Deps, address: Addr, height: u64) -> StdResult<Uint128> {
    let mut power = Uint256::zero(); // Use Uint256 to avoid overflow
    let bonded_vals = BONDED_VALIDATORS
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

                let delegation_power_256 = shares_256
                    .checked_mul(total_tokens_256)?
                    .checked_div(total_shares_256)?;

                power = power.checked_add(delegation_power_256)?;
            }
        }
    }

    // Convert back to Uint128 safely
    let power_128 = Uint128::try_from(power)?;

    Ok(power_128)
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: Addr,
    height: Option<u64>,
) -> StdResult<Uint128> {
    let height = height.unwrap_or(env.block.height);

    if let Some(true) = BLACKLISTED_ADDRESSES.may_load(deps.storage, address.clone())? {
        return Ok(Uint128::zero());
    }

    let power = calculate_voting_power(deps, address, height)?;

    Ok(power)
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<Uint128> {
    let height = height.unwrap_or(env.block.height);

    let bonded_vals = BONDED_VALIDATORS
        .may_load_at_height(deps.storage, height)?
        .unwrap_or_default()
        .into_iter()
        .collect::<HashSet<_>>();

    if bonded_vals.is_empty() {
        return Ok(Uint128::zero());
    }

    // calc total vp as usual
    let total_power: Uint128 = bonded_vals
        .into_iter()
        .map(|valoper_addr| {
            VALIDATORS.may_load_at_height(deps.storage, &Addr::unchecked(valoper_addr), height)
        })
        .collect::<StdResult<Vec<Option<Validator>>>>()?
        .into_iter()
        .map(|m| m.map(|v| v.total_tokens).unwrap_or_default())
        .sum();

    // sum voting power of blacklisted addresses
    let mut blacklisted_power = Uint128::zero();
    for blacklisted_addr in BLACKLISTED_ADDRESSES.keys(deps.storage, None, None, Order::Ascending) {
        let addr = blacklisted_addr?;
        blacklisted_power =
            blacklisted_power.checked_add(calculate_voting_power(deps, addr, height)?)?;
    }

    // subtr blacklisted voting power
    let net_power = total_power.checked_sub(blacklisted_power)?;

    Ok(net_power)
}

pub fn query_dao(deps: Deps) -> StdResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    to_json_binary(&dao)
}

pub fn query_name(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config.name)
}

pub fn query_description(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config.description)
}

pub fn query_list_bonders(
    _deps: Deps,
    _start_after: Option<String>,
    _limit: Option<u32>,
) -> StdResult<Binary> {
    Err(StdError::generic_err("Bonding is disabled"))
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
            msg: to_json_binary(&ProxyInfoExecute::UpdateStake {
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
                msg: to_json_binary(&ProxyInfoExecute::Slashing {})?,
                funds: vec![],
            };

            // Use submsg because we want to ignore possible errors here.
            // This contract should be errorless no matter what.
            resp.add_submessage(SubMsg::reply_on_error(slashing_msg, reason))
        } else {
            resp
        },
    )
}
