#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{coins, to_json_binary, Addr, BankMsg, Binary, CosmosMsg, Decimal256, DelegationResponse, Deps, DepsMut, Env, MessageInfo, Order, Response, StakingQuery, StdResult, Uint128};
use cw2::set_contract_version;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, Delegation, Validator, CONFIG, DAO, DELEGATIONS, VALIDATORS};

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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bond {} => execute_bond(deps, env, info),
        ExecuteMsg::Unbond { amount } => execute_unbond(deps, env, info, amount),
        ExecuteMsg::UpdateConfig {
            owner,
            name,
            description,
        } => execute_update_config(deps, info, owner, name, description),
    }
}

pub fn execute_bond(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    Err(ContractError::BondingDisabled {})
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
        SudoMsg::AfterValidatorBonded { val_address } => {
            after_validator_bonded(deps, env, val_address)
        }
        SudoMsg::AfterValidatorBeginUnbonding { val_address } => {
            after_validator_begin_unbonding(deps, env, val_address)
        }
        SudoMsg::BeforeValidatorSlashed {
            val_address,
            slashing_fraction,
        } => before_validator_slashed(deps, env, val_address, slashing_fraction),
        SudoMsg::AfterDelegationModified {
            delegator_address,
            val_address,
        } => after_delegation_modified(deps, env, delegator_address, val_address),
        SudoMsg::BeforeDelegationRemoved {
            delegator_address,
            val_address,
        } => before_delegation_removed(deps, env, delegator_address, val_address),
        SudoMsg::AfterValidatorCreated { val_address } => {
            after_validator_created(deps, env, val_address)
        }
        SudoMsg::AfterValidatorRemoved {
            valcons_address,
            val_address,
        } => after_validator_removed(deps, env, valcons_address, val_address),
    }
}

// Functions for each SudoMsg variant

pub(crate) fn after_validator_bonded(
    deps: DepsMut,
    env: Env,
    val_address: String,
) -> Result<Response, ContractError> {
    let validator_addr = deps.api.addr_validate(&val_address)?;

    let mut validator = VALIDATORS.may_load(deps.storage, &validator_addr)?.ok_or(
        ContractError::ValidatorNotFound {
            address: val_address.clone(),
        },
    )?;

    if validator.active {
        return Err(ContractError::ValidatorAlreadyActive {
            address: val_address.clone(),
        });
    }

    validator.active = true;

    let staking_query = StakingQuery::Validator {
        address: val_address.clone(),
    };
    let validator_data: cosmwasm_std::ValidatorResponse =
        deps.querier.query(&staking_query.into())?;

    validator.total_tokens = validator_data.validator.tokens;
    validator.total_shares = validator_data.validator.delegator_shares;

    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "validator_bonded")
        .add_attribute("validator_address", val_address)
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string()))
}

fn after_validator_begin_unbonding(
    deps: DepsMut,
    env: Env,
    val_address: String,
) -> Result<Response, ContractError> {
    let validator_addr = deps.api.addr_validate(&val_address)?;

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

    validator.total_tokens = Uint128::zero();
    validator.total_shares = Uint128::zero();

    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    let delegator_prefix = DELEGATIONS.prefix(&validator_addr);
    let delegators: Vec<(Addr, Delegation)> = delegator_prefix
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    for (delegator_addr, _) in delegators {
        DELEGATIONS.remove(
            deps.storage,
            (&delegator_addr, &validator_addr),
            env.block.height,
        )?;
    }

    Ok(Response::new()
        .add_attribute("action", "validator_unbonded")
        .add_attribute("validator_address", val_address))
}

fn before_validator_slashed(
    deps: DepsMut,
    env: Env,
    val_address: String,
    slashing_fraction: Decimal256,
) -> Result<Response, ContractError> {
    let validator_addr = deps.api.addr_validate(&val_address)?;

    let mut validator = VALIDATORS.load(deps.storage, &validator_addr)?;

    if !validator.bonded {
        return Err(ContractError::ValidatorNotBonded {
            validator: validator_addr.to_string(),
        });
    }

    let mut delegator_updates: Vec<(Addr, Uint128)> = Vec::new();
    for (delegator_key, delegation) in DELEGATIONS
        .prefix(&validator_addr)
        .range(deps.storage, None, None, Order::Ascending)
    {
        let delegator_key = delegator_key?;
        let mut delegation = delegation?;

        let slashed_shares = delegation
            .shares
            .checked_mul(Uint128::from(slashing_fraction * Decimal256::from(1u64)))
            .map_err(|e| ContractError::MathError {
                error: e.to_string(),
            })?;
        let updated_shares = delegation.shares.checked_sub(slashed_shares).map_err(|e| {
            ContractError::MathError {
                error: e.to_string(),
            }
        })?;

        // Update the delegation or remove it if shares are zero
        if updated_shares.is_zero() {
            DELEGATIONS.remove(deps.storage, (&delegator_key, &validator_addr), env.block.height);
        } else {
            delegation.shares = updated_shares;
            DELEGATIONS.save(
                deps.storage,
                (&delegator_key, &validator_addr),
                &delegation,
                env.block.height,
            )?;
        }

        delegator_updates.push((delegator_key, slashed_shares));
    }

    // Update the validator's total shares and tokens
    let total_slashed_shares = validator
        .total_shares
        .checked_mul(Uint128::from(slashing_fraction * Decimal256::from(1u64)))
        .map_err(|e| ContractError::MathError {
            error: e.to_string(),
        })?;
    validator.total_shares = validator
        .total_shares
        .checked_sub(total_slashed_shares)
        .map_err(|e| ContractError::MathError {
            error: e.to_string(),
        })?;
    validator.total_tokens = validator.total_shares; // Assumes shares directly correlate with tokens

    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "before_validator_slashed")
        .add_attribute("validator", validator_addr.to_string())
        .add_attribute("slashing_fraction", slashing_fraction.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string())
        .add_attribute("delegator_updates", format!("{:?}", delegator_updates)))
}


pub(crate) fn after_delegation_modified(
    deps: DepsMut,
    env: Env,
    delegator_address: String,
    val_address: String,
) -> Result<Response, ContractError> {
    let delegator = deps.api.addr_validate(&delegator_address)?;
    let validator_addr = deps.api.addr_validate(&val_address)?;

    let query = StakingQuery::Delegation {
        delegator: delegator.clone().to_string(),
        validator: validator_addr.clone().to_string(),
    };

    let query_response: Option<DelegationResponse> = deps.querier.query(&query.into())?;
    let delegation = query_response
        .and_then(|r| r.delegation)
        .ok_or_else(|| ContractError::DelegationNotFound {
            delegator: delegator.to_string(),
            validator: validator_addr.to_string(),
        })?;

    let mut validator = VALIDATORS.load(deps.storage, &validator_addr)?;

    let existing_delegation = DELEGATIONS
        .may_load(deps.storage, (&delegator, &validator_addr))?
        .unwrap_or(Delegation {
            delegator_address: delegator.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::zero(),
        });

    let new_shares = delegation.amount.amount;

    // Determine whether this is an increase or a partial decrease
    if new_shares > existing_delegation.shares {
        // Handle increase (new delegation or redelegation to this validator)
        let added_shares = new_shares
            .checked_sub(existing_delegation.shares)
            .map_err(|e| ContractError::MathError {
                error: e.to_string(),
            })?;
        validator.total_shares = validator
            .total_shares
            .checked_add(added_shares)
            .map_err(|e| ContractError::MathError {
                error: e.to_string(),
            })?;
        validator.total_tokens = validator
            .total_tokens
            .checked_add(added_shares)
            .map_err(|e| ContractError::MathError {
                error: e.to_string(),
            })?;
    } else if new_shares < existing_delegation.shares {
        // Handle partial withdrawal from this validator
        let removed_shares = existing_delegation
            .shares
            .checked_sub(new_shares)
            .map_err(|e| ContractError::MathError {
                error: e.to_string(),
            })?;
        validator.total_shares = validator
            .total_shares
            .checked_sub(removed_shares)
            .map_err(|e| ContractError::MathError {
                error: e.to_string(),
            })?;
        validator.total_tokens = validator
            .total_tokens
            .checked_sub(removed_shares)
            .map_err(|e| ContractError::MathError {
                error: e.to_string(),
            })?;
    }

    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    let updated_delegation = Delegation {
        delegator_address: delegator.clone(),
        validator_address: validator_addr.clone(),
        shares: new_shares,
    };
    DELEGATIONS.save(
        deps.storage,
        (&delegator, &validator_addr),
        &updated_delegation,
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "after_delegation_modified")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("validator", validator_addr.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string()))
}


pub(crate) fn before_delegation_removed(
    deps: DepsMut,
    env: Env,
    delegator_address: String,
    val_address: String,
) -> Result<Response, ContractError> {
    let delegator = deps.api.addr_validate(&delegator_address)?;
    let validator_addr = deps.api.addr_validate(&val_address)?;

    let mut validator = VALIDATORS.load(deps.storage, &validator_addr)?;

    let existing_delegation = DELEGATIONS
        .may_load(deps.storage, (&delegator, &validator_addr))?
        .ok_or_else(|| ContractError::DelegationNotFound {
            delegator: delegator.to_string(),
            validator: validator_addr.to_string(),
        })?;

    // Reduce the validator's shares and tokens to 0 (since this is a full withdrawal)
    validator.total_shares = validator
        .total_shares
        .checked_sub(existing_delegation.shares)
        .map_err(|e| ContractError::MathError {
            error: e.to_string(),
        })?;
    validator.total_tokens = validator
        .total_tokens
        .checked_sub(existing_delegation.shares)
        .map_err(|e| ContractError::MathError {
            error: e.to_string(),
        })?;

    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    DELEGATIONS.remove(deps.storage, (&delegator, &validator_addr), env.block.height);

    Ok(Response::new()
        .add_attribute("action", "before_delegation_removed")
        .add_attribute("delegator", delegator.to_string())
        .add_attribute("validator", validator_addr.to_string())
        .add_attribute("total_shares", validator.total_shares.to_string())
        .add_attribute("total_tokens", validator.total_tokens.to_string()))
}

pub(crate) fn after_validator_removed(
    deps: DepsMut,
    env: Env,
    valcons_address: String,
    val_address: String,
) -> Result<Response, ContractError> {
    let validator_addr = deps.api.addr_validate(&val_address)?;

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

    // reset the validator's tokens and shares to 0?
    validator.total_tokens = Uint128::zero();
    validator.total_shares = Uint128::zero();

    VALIDATORS.save(deps.storage, &validator_addr, &validator, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "validator_disabled")
        .add_attribute("valcons_address", valcons_address)
        .add_attribute("validator_address", val_address))
}

pub(crate) fn after_validator_created(
    deps: DepsMut,
    env: Env,
    val_address: String,
) -> Result<Response, ContractError> {
    let validator_addr = deps.api.addr_validate(&val_address)?;

    if let Some(mut existing_validator) = VALIDATORS.may_load(deps.storage, &validator_addr)? {
        if existing_validator.active {
            return Err(ContractError::ValidatorAlreadyExists {
                address: val_address,
            });
        }

        existing_validator.active = true;
        VALIDATORS.save(
            deps.storage,
            &validator_addr,
            &existing_validator,
            env.block.height,
        )?;

        return Ok(Response::new()
            .add_attribute("action", "validator_enabled")
            .add_attribute("validator_address", val_address));
    }

    let new_validator = Validator {
        address: validator_addr.clone(),
        bonded: false,                 // Initially not bonded
        total_tokens: Uint128::zero(), // No tokens delegated yet
        total_shares: Uint128::zero(), // No shares yet
        active: true,                  // Set to active as it is newly created
    };

    VALIDATORS.save(
        deps.storage,
        &validator_addr,
        &new_validator,
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "validator_created")
        .add_attribute("validator_address", val_address))
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
        QueryMsg::ListBonders { start_after, limit } => {
            todo!()
        }
        QueryMsg::BondingStatus { address, height } => todo!(),
    }
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    let height = height.unwrap_or(env.block.height);
    let address = deps.api.addr_validate(&address)?;

    let mut power = Uint128::zero();
    for val_addr_r in VALIDATORS.keys(deps.storage, None, None, Order::Ascending) {
        let val_addr = val_addr_r?;
        if let Some(val) = VALIDATORS.may_load_at_height(deps.storage, &val_addr, height)? {
            if val.bonded {
                if let Some(delegation) =
                    DELEGATIONS.may_load_at_height(deps.storage, (&address, &val_addr), height)?
                {
                    let delegation_power = delegation
                        .shares
                        .checked_mul(val.total_tokens)?
                        .checked_div(val.total_shares)?;
                    power = power.checked_add(delegation_power)?;
                }
            }
        }
    }
    Ok(VotingPowerAtHeightResponse { power, height })
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let height = height.unwrap_or(env.block.height);
    let mut power = Uint128::zero();
    for k in VALIDATORS.keys(deps.storage, None, None, Order::Ascending) {
        if let Some(val) = VALIDATORS.may_load_at_height(deps.storage, &k?, height)? {
            if val.bonded {
                power = power.checked_add(val.total_tokens)?;
            }
        }
    }

    Ok(TotalPowerAtHeightResponse { power, height })
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
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config.name)
}

pub fn query_description(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config.description)
}

pub fn query_list_bonders(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
