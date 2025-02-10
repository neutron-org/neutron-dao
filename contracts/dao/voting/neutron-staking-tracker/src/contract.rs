use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use crate::state::{
    Config, Delegation, Validator, BLACKLISTED_ADDRESSES, CONFIG, DAO, DELEGATIONS, VALIDATORS,
};
use std::ops::Mul;

use bech32::{encode, Bech32, Hrp};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult, Uint128, Uint256,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use neutron_std::types::cosmos::staking::v1beta1::{QueryValidatorResponse, StakingQuerier};
use prost::Message;
use std::str::FromStr;

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-staking-tracker";
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

    let config = Config {
        name: msg.name,
        description: msg.description,
        owner,
        denom: msg.denom,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("name", config.name)
        .add_attribute("description", config.description)
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
            owner,
            name,
            description,
        } => execute_update_config(deps, info, owner, name, description),
        ExecuteMsg::AddToBlacklist { addresses } => execute_add_to_blacklist(deps, info, addresses),
        ExecuteMsg::RemoveFromBlacklist { addresses } => {
            execute_remove_from_blacklist(deps, info, addresses)
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

pub fn execute_add_to_blacklist(
    deps: DepsMut,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    for address in &addresses {
        let addr = deps.api.addr_validate(address)?;
        BLACKLISTED_ADDRESSES.save(deps.storage, addr, &true)?;
    }

    Ok(Response::new()
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

    for address in &addresses {
        let addr = deps.api.addr_validate(address)?;
        BLACKLISTED_ADDRESSES.remove(deps.storage, addr);
    }

    Ok(Response::new()
        .add_attribute("action", "remove_from_blacklist")
        .add_attribute("removed_addresses", format!("{:?}", addresses)))
}

pub fn execute_unbond(
    _: DepsMut,
    _: Env,
    _: MessageInfo,
    _: Uint128,
) -> Result<Response, ContractError> {
    Err(ContractError::DirectUnbondingDisabled {})
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
    new_name: String,
    new_description: String,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = deps.api.addr_validate(&new_owner)?;

    config.owner = new_owner;
    config.name = new_name;
    config.description = new_description;
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute("owner", config.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::AfterValidatorBonded {
            cons_addr,
            val_addr,
        } => after_validator_bonded(deps, env, cons_addr, val_addr),
        SudoMsg::AfterValidatorRemoved {
            cons_addr,
            val_addr,
        } => after_validator_removed(deps, env, cons_addr, val_addr),
        SudoMsg::AfterValidatorCreated { val_addr } => after_validator_created(deps, env, val_addr),
        SudoMsg::AfterValidatorBeginUnbonding {
            cons_addr,
            val_addr,
        } => after_validator_begin_unbonding(deps, env, cons_addr, val_addr),
        SudoMsg::BeforeValidatorModified { val_addr } => {
            before_validator_modified(deps, env, val_addr)
        }
        SudoMsg::BeforeDelegationCreated { del_addr, val_addr } => {
            before_delegation_created(deps, env, del_addr, val_addr)
        }
        SudoMsg::AfterDelegationModified { del_addr, val_addr } => {
            after_delegation_modified(deps, env, del_addr, val_addr)
        }
        SudoMsg::BeforeValidatorSlashed { val_addr, fraction } => {
            before_validator_slashed(deps, env, val_addr, fraction)
        }
    }
}

pub(crate) fn after_validator_bonded(
    deps: DepsMut,
    env: Env,
    valcons_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let valcons_addr = Addr::unchecked(&valcons_address);
    let valoper_addr = Addr::unchecked(&valoper_address);

    let querier = StakingQuerier::new(&deps.querier);

    // Query the latest validator state from the chain
    let validator_data: QueryValidatorResponse = querier
        .validator(valoper_address.clone())
        .map_err(|_| ContractError::ValidatorQueryFailed {
            address: valoper_address.clone(),
        })?;

    let validator_info = validator_data
        .validator
        .ok_or(ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        })?;

    let total_tokens =
        Uint128::from_str(&validator_info.tokens).map_err(|_| ContractError::InvalidTokenData {
            address: valoper_address.clone(),
        })?;

    let total_shares =
        validator_info
            .delegator_shares
            .parse()
            .map_err(|_| ContractError::InvalidTokenData {
                address: valoper_address.clone(),
            })?;

    // Load validator or initialize if missing
    let mut validator = VALIDATORS
        .may_load(deps.storage, &valoper_addr)?
        .unwrap_or(Validator {
            cons_address: valcons_addr.clone(),
            oper_address: valoper_addr.clone(),
            bonded: false,
            total_tokens: Uint128::zero(),
            total_shares: Uint128::zero(),
            active: true,
        });

    // Update validator state
    validator.bonded = true;
    validator.total_tokens = total_tokens;
    validator.total_shares = total_shares;

    // Save updated validator using valoper as the key
    VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "validator_bonded")
        .add_attribute("valcons_address", valcons_address)
        .add_attribute("valoper_address", valoper_address)
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string()))
}

pub(crate) fn before_validator_modified(
    deps: DepsMut,
    env: Env,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let valoper_addr = Addr::unchecked(&valoper_address);

    let querier = StakingQuerier::new(&deps.querier);

    let validator_data: QueryValidatorResponse = querier
        .validator(valoper_address.clone())
        .map_err(|_| ContractError::ValidatorQueryFailed {
            address: valoper_address.clone(),
        })?;

    let validator_info = validator_data
        .validator
        .ok_or(ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        })?;

    let total_tokens =
        Uint128::from_str(&validator_info.tokens).map_err(|_| ContractError::InvalidTokenData {
            address: valoper_address.clone(),
        })?;

    let total_shares =
        validator_info
            .delegator_shares
            .parse()
            .map_err(|_| ContractError::InvalidTokenData {
                address: valoper_address.clone(),
            })?;

    let mut validator = match VALIDATORS.may_load(deps.storage, &valoper_addr)? {
        Some(existing_validator) => existing_validator,
        None => {
            return Err(ContractError::ValidatorNotFound {
                address: valoper_address.clone(),
            })
        }
    };

    // Update validator state
    validator.total_tokens = total_tokens;
    validator.total_shares = total_shares;

    // Save updated validator information using `valoper` as primary key
    VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "before_validator_modified")
        .add_attribute("valoper_address", valoper_address)
        .add_attribute("cons_address", validator.cons_address.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string()))
}

pub(crate) fn before_delegation_created(
    _deps: DepsMut,
    _env: Env,
    _delegator_address: String,
    _valoper_address: String,
) -> Result<Response, ContractError> {
    // No action required as AfterDelegationModified covers delegation creation and modification
    Ok(Response::new().add_attribute("action", "before_delegation_created"))
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

    let slashed_tokens_uint128: Uint128 =
        slashed_tokens
            .try_into()
            .map_err(|_| ContractError::MathError {
                error: format!(
                    "Failed to convert slashed tokens ({}) to Uint128",
                    slashed_tokens
                ),
            })?;

    // Ensure tokens are reduced but not negative
    validator.total_tokens = validator
        .total_tokens
        .checked_sub(slashed_tokens_uint128)
        .map_err(|_| ContractError::MathError {
            error: format!(
                "Slashed tokens ({}) exceed total tokens ({})",
                slashed_tokens_uint128, validator.total_tokens
            ),
        })?;

    // Save updated validator state
    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "before_validator_slashed")
        .add_attribute("valoper_address", valoper_address)
        .add_attribute("cons_address", validator.cons_address.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("slashing_fraction", slashing_fraction.to_string()))
}

pub(crate) fn after_validator_begin_unbonding(
    deps: DepsMut,
    env: Env,
    valcons_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let valoper_addr = Addr::unchecked(valoper_address.clone());

    // Load validator by valoper_address
    let mut validator = VALIDATORS.may_load(deps.storage, &valoper_addr)?.ok_or(
        ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        },
    )?;

    if !validator.bonded {
        return Err(ContractError::ValidatorNotBonded {
            validator: valoper_address.clone(),
        });
    }

    // Mark validator as unbonded
    validator.bonded = false;
    validator.cons_address = Addr::unchecked(valcons_address);

    // Save updated validator state
    VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "after_validator_begin_unbonding")
        .add_attribute("valoper_address", valoper_address)
        .add_attribute("cons_address", validator.cons_address.to_string()) // Still stored inside
        .add_attribute("unbonding_start_height", env.block.height.to_string()))
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

    // Load validator by `valoper_address`
    let mut validator = VALIDATORS.load(deps.storage, &valoper_addr)?;

    // this means undelegation happened, we can't query a validator from the state because it's not updated yet
    // we need to do calculations of new tokens and shares in a validator manually

    // https://github.com/neutron-org/cosmos-sdk/blob/83295e7c1380071cb9a0f405442d06acf387228c/x/staking/keeper/delegation.go#L1048
    // https://github.com/neutron-org/cosmos-sdk/blob/83295e7c1380071cb9a0f405442d06acf387228c/x/staking/keeper/validator.go#L148
    if actual_shares < previous_shares {
        let undelegated_shares = previous_shares - actual_shares;

        validator.remove_del_shares(undelegated_shares)?;
    } else {
        // Query validator data to get the latest **total tokens & shares**
        let validator_data = querier
            .validator(valoper_address.clone())
            .map_err(|_| ContractError::ValidatorQueryFailed {
                address: valoper_address.clone(),
            })?
            .validator
            .ok_or_else(|| ContractError::ValidatorNotFound {
                address: valoper_address.clone(),
            })?;

        validator.total_shares =
            Uint128::from_str(&validator_data.delegator_shares).map_err(|_| {
                ContractError::InvalidSharesFormat {
                    shares_str: validator_data.delegator_shares.clone(),
                }
            })?;

        validator.total_tokens = Uint128::from_str(&validator_data.tokens).map_err(|_| {
            ContractError::InvalidTokenData {
                address: valoper_address.clone(),
            }
        })?;
    }

    // Save updated validator state
    VALIDATORS.save(deps.storage, &valoper_addr, &validator, env.block.height)?;

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

    Ok(Response::new()
        .add_attribute("action", "after_delegation_modified")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("cons_address", validator.cons_address.to_string()) // Still stored inside Validator
        .add_attribute("valoper_address", valoper_address.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("delegation_shares", actual_shares.to_string()))
}

pub(crate) fn after_validator_removed(
    deps: DepsMut,
    env: Env,
    valoper_address: String,
    valcons_address: String,
) -> Result<Response, ContractError> {
    let validator_addr = Addr::unchecked(&valoper_address);

    // Load validator using `valoper_address` as the primary key
    let mut validator = VALIDATORS.may_load(deps.storage, &validator_addr)?.ok_or(
        ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        },
    )?;

    if !validator.active {
        return Err(ContractError::ValidatorNotActive {
            address: valoper_address.clone(),
        });
    }

    // Mark validator as inactive (soft removal)
    validator.active = false;

    // Save updated validator state
    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "after_validator_removed")
        .add_attribute("valoper_address", valoper_address)
        .add_attribute("valcons_address", valcons_address))
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
        .validator(valoper_address.clone())
        .map_err(|_| ContractError::ValidatorQueryFailed {
            address: valoper_address.clone(),
        })?
        .validator
        .ok_or_else(|| ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        })?;

    let total_tokens =
        Uint128::from_str(&validator_data.tokens).map_err(|_| ContractError::InvalidTokenData {
            address: valoper_address.clone(),
        })?;

    let total_shares = Uint128::from_str(&validator_data.delegator_shares).map_err(|_| {
        ContractError::InvalidTokenData {
            address: valoper_address.clone(),
        }
    })?;

    let new_validator = Validator {
        cons_address: Addr::unchecked(""),
        oper_address: valoper_addr.clone(),
        bonded: false,
        total_tokens,
        total_shares,
        active: true,
    };

    // Use `valoper_address` as the primary key for storage
    VALIDATORS.save(
        deps.storage,
        &valoper_addr, // Primary key is now `valoper`
        &new_validator,
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "validator_created")
        .add_attribute("operator_address", valoper_address)
        .add_attribute("total_tokens", total_tokens.to_string())
        .add_attribute("total_shares", total_shares.to_string()))
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

/// Converts a `valoper` address into a `valcons` address
pub fn get_consensus_address(deps: Deps, valoper_address: String) -> Result<String, ContractError> {
    let querier = StakingQuerier::new(&deps.querier);

    // Query validator details from the chain
    let validator_data = querier.validator(valoper_address.clone()).map_err(|_| {
        ContractError::ValidatorQueryFailed {
            address: valoper_address.clone(),
        }
    })?;

    let consensus_pubkey_any = validator_data
        .validator
        .ok_or(ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        })?
        .consensus_pubkey
        .ok_or(ContractError::NoPubKey {
            address: valoper_address.clone(),
        })?;

    // Decode consensus public key from Protobuf Any
    let public_key = neutron_std::types::cosmos::crypto::ed25519::PubKey::decode(
        consensus_pubkey_any.value.as_ref(),
    )
    .map_err(|_| ContractError::InvalidConsensusKey)?;

    let hrp = Hrp::parse("neutronvalcons").map_err(|_| ContractError::InvalidConsensusKey)?;
    let key_bytes: &[u8] = &public_key.key;
    let encoded = encode::<Bech32>(hrp, key_bytes)
        .map_err(|_| ContractError::InvalidConsensusKey)?
        .to_string();
    Ok(encoded)
}

/// Calculates the voting power of a delegator at a specific block height.
///
/// Uses `Uint256` for intermediate calculations to avoid precision loss and overflow,
/// then converts the final result back to `Uint128`.
///
pub fn calculate_voting_power(deps: Deps, address: Addr, height: u64) -> StdResult<Uint128> {
    let mut power = Uint256::zero(); // Use Uint256 to avoid overflow

    for val_oper_address in VALIDATORS.keys(deps.storage, None, None, Order::Ascending) {
        if let Some(validator) =
            VALIDATORS.may_load_at_height(deps.storage, &val_oper_address?, height)?
        {
            if validator.bonded {
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
    }

    // Convert back to Uint128 safely
    let power_128 = Uint128::try_from(power)
        .map_err(|_| StdError::generic_err("Overflow: Uint256 to Uint128 conversion failed"))?;

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

    // calc total vp as usual
    let mut total_power = Uint128::zero();
    for k in VALIDATORS.keys(deps.storage, None, None, Order::Ascending) {
        if let Some(val) = VALIDATORS.may_load_at_height(deps.storage, &k?, height)? {
            if val.bonded {
                total_power = total_power.checked_add(val.total_tokens)?;
            }
        }
    }

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
