#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::must_pay;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use cwd_voting::vault::{BonderBalanceResponse, ListBondersResponse};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, Validator, CONFIG, DAO, DELEGATIONS, VALIDATORS};

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
        SudoMsg::AfterValidatorBonded { val_address } => todo!(),
        SudoMsg::AfterValidatorBeginUnbonding { val_address } => todo!(),
        SudoMsg::BeforeValidatorSlashed {
            val_address,
            slashing_fraction,
        } => todo!(),
        SudoMsg::AfterDelegationModified {
            delegator_address,
            val_address,
        } => todo!(),
        SudoMsg::BeforeDelegationRemoved {
            delegator_address,
            val_address,
        } => todo!(),
        SudoMsg::AfterValidatorCreated { val_address } => todo!(),
        SudoMsg::AfterValidatorRemoved {
            valcons_address,
            val_address,
        } => todo!(),
    }
}

pub fn after_validator_created(
    deps: DepsMut,
    env: Env,
    val_address: String,
) -> Result<Response, ContractError> {
    let addr = deps.api.addr_validate(&val_address)?;
    VALIDATORS.save(
        deps.storage,
        &addr,
        &Validator {
            address: addr,
            bonded: false,
            total_tokens: todo!(), // query total tokens
            total_shares: todo!(), // query total shares
        },
        env.block.height,
    );

    Ok(Response::new()
        .add_attribute("action", "validator_created")
        .add_attribute("address", addr))
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
