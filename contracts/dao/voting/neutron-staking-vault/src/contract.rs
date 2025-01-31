use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use crate::state::{
    Config, Delegation, Validator, BLACKLISTED_ADDRESSES, CONFIG, DAO, DELEGATIONS,
    OPERATOR_TO_CONSENSUS, VALIDATORS,
};

use bech32::{encode, Bech32, Hrp};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use neutron_std::types::cosmos::staking::v1beta1::{QueryValidatorResponse, StakingQuerier};
use prost::Message;
use std::str::FromStr;

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-voting-vault";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = Addr::unchecked(&msg.owner);

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
    env: Env,
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
        let addr = Addr::unchecked(address);
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
        let addr = Addr::unchecked(address);
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

    let new_owner = Addr::unchecked(&new_owner);

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
        SudoMsg::BeforeDelegationSharesModified { del_addr, val_addr } => {
            before_delegation_shares_modified(deps, env, del_addr, val_addr)
        }
        SudoMsg::BeforeDelegationRemoved { del_addr, val_addr } => {
            before_delegation_removed(deps, env, del_addr, val_addr)
        }
        SudoMsg::AfterDelegationModified { del_addr, val_addr } => {
            after_delegation_modified(deps, env, del_addr, val_addr)
        }
        SudoMsg::BeforeValidatorSlashed { val_addr, fraction } => {
            before_validator_slashed(deps, env, val_addr, fraction)
        }
        SudoMsg::AfterUnbondingInitiated { id } => {
            after_unbonding_initiated(deps, env, u64::from(id))
        }
    }
}

pub(crate) fn before_delegation_shares_modified(
    _deps: DepsMut,
    _env: Env,
    _delegator_address: String,
    _valoper_address: String,
) -> Result<Response, ContractError> {
    // No action required as AfterDelegationSharesModified covers delegation creation and modification
    Ok(Response::new().add_attribute("action", "before_delegation_shares_modified"))
}

pub(crate) fn after_unbonding_initiated(
    _deps: DepsMut,
    _env: Env,
    _id: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "after_unbonding_initiated"))
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
    let validator_data: QueryValidatorResponse = match querier.validator(valoper_address.clone()) {
        Ok(data) => data,
        Err(e) => {
            return Err(ContractError::ValidatorQueryFailed {
                address: valoper_address.clone(),
            });
        }
    };

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

    // Load validator, or initialize if missing
    let mut validator = VALIDATORS
        .may_load(deps.storage, &valcons_addr)?
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

    // Save updated validator information
    VALIDATORS.save(deps.storage, &valcons_addr, &validator, env.block.height)?;

    // Save operator-to-consensus mapping
    OPERATOR_TO_CONSENSUS.save(deps.storage, &valoper_addr, &valcons_addr)?;

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

    let valcons_address =
        get_consensus_address(deps.as_ref(), valoper_address.clone()).map_err(|_| {
            ContractError::ValidatorQueryFailed {
                address: valoper_address.clone(),
            }
        })?;

    let valcons_addr = Addr::unchecked(&valcons_address);
    let valoper_addr = Addr::unchecked(&valoper_address);

    let mut validator = VALIDATORS
        .may_load(deps.storage, &valcons_addr)?
        .unwrap_or(Validator {
            cons_address: valcons_addr.clone(),
            oper_address: valoper_addr.clone(),
            bonded: false,
            total_tokens: Uint128::zero(),
            total_shares: Uint128::zero(),
            active: true,
        });

    // Update operator address if it has changed
    if validator.oper_address != valoper_addr {
        validator.oper_address = valoper_addr;
    }

    // Save updated validator information
    VALIDATORS.save(deps.storage, &valcons_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "before_validator_modified")
        .add_attribute("valcons_address", valcons_address)
        .add_attribute("valoper_address", valoper_address))
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
    let querier = StakingQuerier::new(&deps.querier);

    // Retrieve validator consensus address
    let valcons_address = get_consensus_address(deps.as_ref(), valoper_address.clone())?;
    let valcons_addr = Addr::unchecked(valcons_address);

    // Load validator state
    let mut validator = VALIDATORS.may_load(deps.storage, &valcons_addr)?.ok_or(
        ContractError::ValidatorNotFound {
            address: valcons_addr.to_string(),
        },
    )?;

    let delegations_response = querier
        .validator_delegations(valoper_address.clone(), None)
        .map_err(|err| {
            println!(
                "❌ Delegation query failed for validator {}: {:?}",
                valoper_address, err
            );
            ContractError::DelegationQueryFailed {
                validator: valoper_address.clone(),
            }
        })?;

    // Overwrite delegations with latest state from staking module
    for delegation in delegations_response.delegation_responses.iter() {
        let delegator_addr =
            Addr::unchecked(&delegation.delegation.as_ref().unwrap().delegator_address);
        let updated_shares = Uint128::from_str(&delegation.delegation.as_ref().unwrap().shares)
            .map_err(|_| ContractError::InvalidTokenData {
                address: delegator_addr.to_string(),
            })?;

        // Save the **new** delegation state (overwriting old one)
        DELEGATIONS.save(
            deps.storage,
            (&delegator_addr, &valcons_addr),
            &Delegation {
                delegator_address: delegator_addr.clone(),
                validator_address: valcons_addr.clone(),
                shares: updated_shares,
            },
            env.block.height,
        )?;
    }

    // **Query validator data again** to get latest tokens & shares after slashing
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

    validator.total_tokens =
        Uint128::from_str(&validator_info.tokens).map_err(|_| ContractError::InvalidTokenData {
            address: valoper_address.clone(),
        })?;

    validator.total_shares =
        validator_info
            .delegator_shares
            .parse()
            .map_err(|_| ContractError::InvalidTokenData {
                address: valoper_address.clone(),
            })?;

    // Save updated validator state
    VALIDATORS.save(deps.storage, &valcons_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "before_validator_slashed")
        .add_attribute("valcons_address", valcons_addr.to_string())
        .add_attribute("valoper_address", valoper_address)
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
    let valcons_addr = Addr::unchecked(valcons_address);

    let mut validator = VALIDATORS.may_load(deps.storage, &valcons_addr)?.ok_or(
        ContractError::ValidatorNotFound {
            address: valcons_addr.to_string(),
        },
    )?;

    if !validator.bonded {
        return Err(ContractError::ValidatorNotBonded {
            validator: valcons_addr.to_string(),
        });
    }

    validator.bonded = false;
    validator.oper_address = Addr::unchecked(&valoper_address); // Update the latest valoper address

    VALIDATORS.save(deps.storage, &valcons_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "after_validator_begin_unbonding")
        .add_attribute("valcons_address", valcons_addr.to_string())
        .add_attribute("valoper_address", validator.oper_address.to_string())
        .add_attribute("unbonding_start_height", env.block.height.to_string()))
}

pub(crate) fn after_delegation_modified(
    deps: DepsMut,
    env: Env,
    delegator_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let delegator = Addr::unchecked(&delegator_address);

    // Retrieve consensus address using stored map or query
    let valcons_address = OPERATOR_TO_CONSENSUS
        .may_load(deps.storage, &Addr::unchecked(&valoper_address))?
        .unwrap_or_else(|| {
            Addr::unchecked(get_consensus_address(deps.as_ref(), valoper_address.clone()).unwrap())
        });

    let validator_addr = Addr::unchecked(valcons_address.clone());

    let querier = StakingQuerier::new(&deps.querier);

    // Query **current delegation state** from the chain
    let delegation_info = querier.delegation(delegator_address.clone(), valoper_address.clone())?;

    // Extract the actual delegation shares
    let actual_shares = delegation_info
        .delegation_response
        .and_then(|resp| resp.delegation)
        .map(|del| Uint128::from_str(&del.shares))
        .transpose()
        .map_err(|_| ContractError::InvalidSharesFormat)?
        .unwrap_or(Uint128::zero()); // Default to zero if delegation does not exist

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

    let total_validator_shares = Uint128::from_str(&validator_data.delegator_shares)
        .map_err(|_| ContractError::InvalidSharesFormat)?;

    let total_validator_tokens =
        Uint128::from_str(&validator_data.tokens).map_err(|_| ContractError::InvalidTokenData {
            address: valoper_address.clone(),
        })?;

    // Update validator state in contract
    let mut validator = VALIDATORS.load(deps.storage, &validator_addr)?;
    validator.total_shares = total_validator_shares;
    validator.total_tokens = total_validator_tokens;

    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    // **Ensure delegation is correctly overwritten with actual shares**
    DELEGATIONS.save(
        deps.storage,
        (&delegator, &validator_addr),
        &Delegation {
            delegator_address: delegator.clone(),
            validator_address: validator_addr.clone(),
            shares: actual_shares, // **Ensure correct shares are stored**
        },
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "after_delegation_modified")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("valcons_address", valcons_address.to_string())
        .add_attribute("valoper_address", valoper_address.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("delegation_shares", actual_shares.to_string()))
}

pub(crate) fn before_delegation_removed(
    deps: DepsMut,
    env: Env,
    delegator_address: String,
    valoper_address: String,
) -> Result<Response, ContractError> {
    let delegator = Addr::unchecked(&delegator_address);

    // Fetch consensus address from state, fallback to query if missing
    let valcons_address = OPERATOR_TO_CONSENSUS
        .may_load(deps.storage, &Addr::unchecked(&valoper_address))?
        .unwrap_or_else(|| {
            Addr::unchecked(get_consensus_address(deps.as_ref(), valoper_address.clone()).unwrap())
        });

    let valoper_addr = Addr::unchecked(&valoper_address);

    // Load validator state using `valcons`
    let mut validator = VALIDATORS.load(deps.storage, &valcons_address)?;

    // Load existing delegation using `valoper`
    let existing_delegation = DELEGATIONS
        .may_load(deps.storage, (&delegator, &valoper_addr))? // Now using `valoper`
        .ok_or_else(|| ContractError::DelegationNotFound {
            delegator: delegator.to_string(),
            validator: valoper_addr.to_string(), // Use `valoper` in error message
        })?;

    // Query actual delegation amount from staking module
    let querier = StakingQuerier::new(&deps.querier);
    let delegation_info = querier
        .delegation(delegator_address.clone(), valoper_address.clone())?
        .delegation_response
        .ok_or_else(|| ContractError::DelegationBalanceNotFound {
            delegator: delegator.to_string(),
            validator: valoper_addr.to_string(),
        })?;

    let actual_shares = Uint128::from_str(&delegation_info.delegation.unwrap().shares)?;

    // Query updated validator data
    let validator_data = querier
        .validator(valoper_address.clone())
        .map_err(|_| ContractError::ValidatorQueryFailed {
            address: valoper_address.clone(),
        })?
        .validator
        .ok_or_else(|| ContractError::ValidatorNotFound {
            address: valoper_address.clone(),
        })?;

    validator.total_shares = Uint128::from_str(&validator_data.clone().delegator_shares)?;
    validator.total_tokens = Uint128::from_str(&validator_data.tokens)?;

    // Save updated validator state
    VALIDATORS.save(deps.storage, &valcons_address, &validator, env.block.height)?;

    //  Overwrite delegation using `valoper`
    DELEGATIONS.save(
        deps.storage,
        (&delegator, &valoper_addr), // Ensure `valoper` is used
        &Delegation {
            delegator_address: delegator.clone(),
            validator_address: valoper_addr.clone(),
            shares: actual_shares,
        },
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "before_delegation_removed")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("validator", valoper_addr.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string()))
}

pub(crate) fn after_validator_removed(
    deps: DepsMut,
    env: Env,
    valcons_address: String,
    val_address: String,
) -> Result<Response, ContractError> {
    let validator_addr = Addr::unchecked(&val_address);
    let valcon_addr = Addr::unchecked(valcons_address.clone());

    let mut validator = VALIDATORS.may_load(deps.storage, &validator_addr)?.ok_or(
        ContractError::ValidatorNotFound {
            address: val_address.clone(),
        },
    )?;

    if !validator.active {
        return Err(ContractError::ValidatorNotActive {
            address: val_address.clone(),
        });
    }

    validator.active = false;

    VALIDATORS.save(deps.storage, &valcon_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "after_validator_removed")
        .add_attribute("valcons_address", valcons_address.to_string())
        .add_attribute("validator_address", val_address))
}

pub(crate) fn after_validator_created(
    deps: DepsMut,
    env: Env,
    valoper_address: String,
) -> Result<Response, ContractError> {
    // Retrieve the consensus address using the helper function
    let cons_address = get_consensus_address(deps.as_ref(), valoper_address.clone())?;

    let validator_addr = Addr::unchecked(&cons_address);

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

    let total_shares =
        validator_data
            .delegator_shares
            .parse()
            .map_err(|_| ContractError::InvalidTokenData {
                address: valoper_address.clone(),
            })?;

    let new_validator = Validator {
        cons_address: validator_addr.clone(),
        oper_address: Addr::unchecked(valoper_address.clone()),
        bonded: false,
        total_tokens,
        total_shares,
        active: true,
    };

    VALIDATORS.save(
        deps.storage,
        &validator_addr,
        &new_validator,
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "validator_created")
        .add_attribute("consensus_address", cons_address)
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
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Addr>> {
    let start = start_after
        .map(|addr| Addr::unchecked(&addr))
        .map(Bound::exclusive); // Convert to exclusive Bound

    let limit = limit.unwrap_or(10) as usize;

    let blacklisted: Vec<Addr> = BLACKLISTED_ADDRESSES
        .keys(deps.storage, start, None, cosmwasm_std::Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(blacklisted)
}

pub fn query_is_address_blacklisted(deps: Deps, address: String) -> StdResult<bool> {
    let addr = Addr::unchecked(&address);
    let is_blacklisted = BLACKLISTED_ADDRESSES
        .may_load(deps.storage, addr)?
        .unwrap_or(false);
    Ok(is_blacklisted)
}

/// Converts a `valoper` address into a `valcons` address
pub fn get_consensus_address(deps: Deps, valoper_address: String) -> Result<String, ContractError> {
    let valoper_addr = Addr::unchecked(valoper_address.clone());

    // First, check if we already have the consensus address stored
    if let Some(stored_cons_address) =
        OPERATOR_TO_CONSENSUS.may_load(deps.storage, &valoper_addr)?
    {
        return Ok(stored_cons_address.to_string());
    }

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

pub fn calculate_voting_power(deps: Deps, address: Addr, height: u64) -> StdResult<Uint128> {
    let mut power = Uint128::zero();

    for val_cons_addr_r in VALIDATORS.keys(deps.storage, None, None, Order::Ascending) {
        let val_cons_addr = val_cons_addr_r?;

        if let Some(validator) =
            VALIDATORS.may_load_at_height(deps.storage, &val_cons_addr, height)?
        {
            if validator.bonded {
                // Use validator's **operator address** to fetch delegations
                let val_oper_addr = validator.oper_address.clone();

                if let Some(delegation) = DELEGATIONS.may_load_at_height(
                    deps.storage,
                    (&address, &val_oper_addr),
                    height,
                )? {
                    let delegation_power = delegation
                        .shares
                        .checked_mul(validator.total_tokens)?
                        .checked_div(validator.total_shares)?;
                    power = power.checked_add(delegation_power)?;
                }
            }
        }
    }

    Ok(power)
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<Uint128> {
    let height = height.unwrap_or(env.block.height);
    let address = Addr::unchecked(&address);

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
