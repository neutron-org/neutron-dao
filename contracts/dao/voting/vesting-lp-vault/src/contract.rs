#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal256, Deps, DepsMut, Env, Fraction, MessageInfo, Response, StdError,
    Uint128,
};
use cw2::set_contract_version;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use serde::Serialize;

use crate::state::{CONFIG, DAO};
use neutron_oracle::voting_power::voting_power_from_lp_tokens;
use neutron_vesting_lp_vault::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    types::Config,
};
use vesting_base::msg::{QueryMsg as VestingLpQueryMsg, QueryMsgHistorical};

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-vesting-lp-vault";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    let config = Config {
        name: msg.name,
        description: msg.description,
        atom_vesting_lp_contract: deps.api.addr_validate(&msg.atom_vesting_lp_contract)?,
        atom_oracle_contract: deps.api.addr_validate(&msg.atom_oracle_contract)?,
        usdc_vesting_lp_contract: deps.api.addr_validate(&msg.usdc_vesting_lp_contract)?,
        usdc_oracle_contract: deps.api.addr_validate(&msg.usdc_oracle_contract)?,
        owner,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("name", config.name)
        .add_attribute("description", config.description)
        .add_attribute("owner", config.owner)
        .add_attribute("atom_vesting_lp_contract", config.atom_vesting_lp_contract)
        .add_attribute("atom_oracle_contract", config.atom_oracle_contract)
        .add_attribute("usdc_vesting_lp_contract", config.usdc_vesting_lp_contract)
        .add_attribute("usdc_oracle_contract", config.usdc_oracle_contract))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Bond {} => execute_bond(deps, env, info),
        ExecuteMsg::Unbond { amount } => execute_unbond(deps, env, info, amount),
        ExecuteMsg::UpdateConfig {
            owner,
            atom_vesting_lp_contract,
            atom_oracle_contract,
            usdc_vesting_lp_contract,
            usdc_oracle_contract,
            name,
            description,
        } => execute_update_config(
            deps,
            info,
            owner,
            atom_vesting_lp_contract,
            atom_oracle_contract,
            usdc_vesting_lp_contract,
            usdc_oracle_contract,
            name,
            description,
        ),
    }
}

pub fn execute_bond(_deps: DepsMut, _env: Env, _info: MessageInfo) -> ContractResult<Response> {
    Err(ContractError::BondingDisabled {})
}

pub fn execute_unbond(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _amount: Uint128,
) -> ContractResult<Response> {
    Err(ContractError::DirectUnbondingDisabled {})
}

#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
    new_atom_vesting_lp_contract: String,
    new_atom_oracle_contract: String,
    new_usdc_vesting_lp_contract: String,
    new_usdc_oracle_contract: String,
    new_name: String,
    new_description: String,
) -> ContractResult<Response> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = deps.api.addr_validate(&new_owner)?;
    let new_atom_vesting_lp_contract = deps.api.addr_validate(&new_atom_vesting_lp_contract)?;
    let new_atom_oracle_contract = deps.api.addr_validate(&new_atom_oracle_contract)?;
    let new_usdc_vesting_lp_contract = deps.api.addr_validate(&new_usdc_vesting_lp_contract)?;
    let new_usdc_oracle_contract = deps.api.addr_validate(&new_usdc_oracle_contract)?;

    config.owner = new_owner;
    config.atom_vesting_lp_contract = new_atom_vesting_lp_contract;
    config.atom_oracle_contract = new_atom_oracle_contract;
    config.usdc_vesting_lp_contract = new_usdc_vesting_lp_contract;
    config.usdc_oracle_contract = new_usdc_oracle_contract;
    config.name = new_name;
    config.description = new_description;
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute("owner", config.owner)
        .add_attribute("atom_vesting_lp_contract", config.atom_vesting_lp_contract)
        .add_attribute("atom_oracle_contract", config.atom_oracle_contract)
        .add_attribute("usdc_vesting_lp_contract", config.usdc_vesting_lp_contract)
        .add_attribute("usdc_oracle_contract", config.usdc_oracle_contract))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::VotingPowerAtHeight { address, height } => Ok(to_binary(
            &query_voting_power_at_height(deps, env, address, height)?,
        )?),
        QueryMsg::TotalPowerAtHeight { height } => {
            Ok(to_binary(&query_total_power_at_height(deps, env, height)?)?)
        }
        QueryMsg::Info {} => query_info(deps),
        QueryMsg::Dao {} => query_dao(deps),
        QueryMsg::Name {} => query_name(deps),
        QueryMsg::Description {} => query_description(deps),
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::ListBonders { start_after, limit } => {
            query_list_bonders(deps, start_after, limit)
        }
        QueryMsg::BondingStatus { height, address } => Ok(to_binary(&query_bonding_status(
            deps, env, height, address,
        )?)?),
    }
}

fn get_voting_power(
    deps: Deps,
    config: &Config,
    height: u64,
    query_msg: &impl Serialize,
) -> ContractResult<Decimal256> {
    let mut voting_power = Decimal256::zero();
    for (vesting_lp, oracle) in [
        (
            &config.atom_vesting_lp_contract,
            &config.atom_oracle_contract,
        ),
        (
            &config.usdc_vesting_lp_contract,
            &config.usdc_oracle_contract,
        ),
    ] {
        voting_power += voting_power_from_lp_tokens(
            deps,
            deps.querier
                .query_wasm_smart::<Option<Uint128>>(vesting_lp, &query_msg)?
                .unwrap_or_default(),
            oracle,
            height,
        )?;
    }
    Ok(voting_power)
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> ContractResult<VotingPowerAtHeightResponse> {
    let config = CONFIG.load(deps.storage)?;
    let height = height.unwrap_or(env.block.height);
    let query_msg = VestingLpQueryMsg::HistoricalExtension {
        msg: QueryMsgHistorical::UnclaimedAmountAtHeight { address, height },
    };

    Ok(VotingPowerAtHeightResponse {
        power: get_voting_power(deps, &config, height, &query_msg)?
            .numerator()
            .try_into()
            .map_err(StdError::from)?,
        height,
    })
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> ContractResult<TotalPowerAtHeightResponse> {
    let config = CONFIG.load(deps.storage)?;
    let height = height.unwrap_or(env.block.height);
    let query_msg = VestingLpQueryMsg::HistoricalExtension {
        msg: QueryMsgHistorical::UnclaimedTotalAmountAtHeight { height },
    };

    Ok(TotalPowerAtHeightResponse {
        power: get_voting_power(deps, &config, height, &query_msg)?
            .numerator()
            .try_into()
            .map_err(StdError::from)?,
        height,
    })
}

pub fn query_info(deps: Deps) -> ContractResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    Ok(to_binary(&cwd_interface::voting::InfoResponse { info })?)
}

pub fn query_dao(deps: Deps) -> ContractResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    Ok(to_binary(&dao)?)
}

pub fn query_name(deps: Deps) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&config.name)?)
}

pub fn query_description(deps: Deps) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&config.description)?)
}

pub fn query_config(deps: Deps) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&config)?)
}

pub fn query_list_bonders(
    _deps: Deps,
    _start_after: Option<String>,
    _limit: Option<u32>,
) -> ContractResult<Binary> {
    Err(ContractError::BondingDisabled {})
}

pub fn query_bonding_status(
    _deps: Deps,
    _env: Env,
    _height: Option<u64>,
    _address: String,
) -> ContractResult<BondingStatusResponse> {
    Err(ContractError::BondingDisabled {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
