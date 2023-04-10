use astroport::vesting::{OrderBy, VestingAccountsResponse};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, Fraction, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use cwd_interface::Admin;

use crate::state::{CONFIG, DAO};
use cwd_voting::vault::{BonderBalanceResponse, ListBondersResponse};
use neutron_lockdrop_vault::voting_power::get_voting_power;
use neutron_lp_vesting_vault::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    types::Config,
};
use vesting_lp::msg::QueryMsg as VestingLpQueryMsg;

pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-lp-vesting-vault";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = match msg.owner {
        Admin::Address { addr } => deps.api.addr_validate(addr.as_str())?,
        Admin::CoreModule {} => info.sender.clone(),
    };
    let manager = msg
        .manager
        .map(|manager| deps.api.addr_validate(&manager))
        .transpose()?;

    let config = Config {
        name: msg.name,
        description: msg.description,
        lp_vesting_contract: deps.api.addr_validate(&msg.lp_vesting_contract)?,
        atom_oracle_contract: deps.api.addr_validate(&msg.atom_oracle_contract)?,
        usdc_oracle_contract: deps.api.addr_validate(&msg.usdc_oracle_contract)?,
        owner,
        manager,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("name", config.name)
        .add_attribute("description", config.description)
        .add_attribute("owner", config.owner)
        .add_attribute("lp_vesting_contract", config.lp_vesting_contract)
        .add_attribute("atom_oracle_contract", config.atom_oracle_contract)
        .add_attribute("usdc_oracle_contract", config.usdc_oracle_contract)
        .add_attribute(
            "manager",
            config
                .manager
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        ))
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
            lp_vesting_contract,
            atom_oracle_contract,
            usdc_oracle_contract,
            manager,
            name,
            description,
        } => execute_update_config(
            deps,
            info,
            owner,
            lp_vesting_contract,
            atom_oracle_contract,
            usdc_oracle_contract,
            manager,
            name,
            description,
        ),
    }
}

pub fn execute_bond(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    Err(ContractError::BondingDisabled {})
}

pub fn execute_unbond(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    Err(ContractError::DirectUnbondingDisabled {})
}

#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
    new_lp_vesting_contract: String,
    new_atom_oracle_contract: String,
    new_usdc_oracle_contract: String,
    new_manager: Option<String>,
    new_name: String,
    new_description: String,
) -> ContractResult<Response> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner && Some(info.sender.clone()) != config.manager {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = deps.api.addr_validate(&new_owner)?;
    let new_lp_vesting_contract = deps.api.addr_validate(&new_lp_vesting_contract)?;
    let new_atom_oracle_contract = deps.api.addr_validate(&new_atom_oracle_contract)?;
    let new_usdc_oracle_contract = deps.api.addr_validate(&new_usdc_oracle_contract)?;
    let new_manager = new_manager
        .map(|new_manager| deps.api.addr_validate(&new_manager))
        .transpose()?;

    if info.sender != config.owner && new_owner != config.owner {
        return Err(ContractError::OnlyOwnerCanChangeOwner {});
    };
    if info.sender != config.owner && new_lp_vesting_contract != config.lp_vesting_contract {
        return Err(ContractError::OnlyOwnerCanChangeLpVestingContract {});
    };

    config.owner = new_owner;
    config.lp_vesting_contract = new_lp_vesting_contract;
    config.atom_oracle_contract = new_atom_oracle_contract;
    config.usdc_oracle_contract = new_usdc_oracle_contract;
    config.manager = new_manager;
    config.name = new_name;
    config.description = new_description;
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("description", config.description)
        .add_attribute("owner", config.owner)
        .add_attribute("lp_vesting_contract", config.lp_vesting_contract)
        .add_attribute("atom_oracle_contract", config.atom_oracle_contract)
        .add_attribute("usdc_oracle_contract", config.usdc_oracle_contract)
        .add_attribute(
            "manager",
            config
                .manager
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VotingPowerAtHeight { address, height } => {
            to_binary(&query_voting_power_at_height(deps, env, address, height)?)
        }
        QueryMsg::TotalPowerAtHeight { height } => {
            to_binary(&query_total_power_at_height(deps, env, height)?)
        }
        QueryMsg::Info {} => query_info(deps),
        QueryMsg::Dao {} => query_dao(deps),
        QueryMsg::Name {} => query_name(deps),
        QueryMsg::Description {} => query_description(deps),
        QueryMsg::GetConfig {} => query_config(deps),
        QueryMsg::ListBonders { start_after, limit } => {
            query_list_bonders(deps, start_after, limit, None)
        }
        QueryMsg::BondingStatus { height, address } => {
            to_binary(&query_bonding_status(deps, env, height, address)?)
        }
    }
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    let config = CONFIG.load(deps.storage)?;
    let height = height.unwrap_or(env.block.height);

    let query_msg = VestingLpQueryMsg::UnclaimedAmountAtHeight { address, height };
    let atom_power = get_voting_power(
        deps,
        &config.lp_vesting_contract,
        &config.atom_oracle_contract,
        &query_msg,
        height,
    )?;
    let usdc_power = get_voting_power(
        deps,
        &config.lp_vesting_contract,
        &config.usdc_oracle_contract,
        &query_msg,
        height,
    )?;

    Ok(VotingPowerAtHeightResponse {
        power: (atom_power + usdc_power).numerator().try_into()?,
        height,
    })
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let config = CONFIG.load(deps.storage)?;
    let height = height.unwrap_or(env.block.height);

    let query_msg = VestingLpQueryMsg::UnclaimedTotalAmountAtHeight { height };
    let atom_power = get_voting_power(
        deps,
        &config.lp_vesting_contract,
        &config.atom_oracle_contract,
        &query_msg,
        height,
    )?;
    let usdc_power = get_voting_power(
        deps,
        &config.lp_vesting_contract,
        &config.usdc_oracle_contract,
        &query_msg,
        height,
    )?;

    Ok(TotalPowerAtHeightResponse {
        power: (atom_power + usdc_power).numerator().try_into()?,
        height,
    })
}

pub fn query_info(deps: Deps) -> StdResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    to_binary(&cwd_interface::voting::InfoResponse { info })
}

pub fn query_dao(deps: Deps) -> StdResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    to_binary(&dao)
}

pub fn query_name(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&config.name)
}

pub fn query_description(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&config.description)
}

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&config)
}

pub fn query_list_bonders(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Binary> {
    // TODO: this method does not adjust LP tokens amount to their corresponding voting power
    let config = CONFIG.load(deps.storage)?;

    let vesting_accounts: VestingAccountsResponse = deps.querier.query_wasm_smart(
        config.lp_vesting_contract,
        &VestingLpQueryMsg::VestingAccounts {
            start_after,
            limit,
            order_by,
        },
    )?;

    let bonders = vesting_accounts
        .vesting_accounts
        .into_iter()
        .map(|vesting_account| BonderBalanceResponse {
            address: vesting_account.address.into_string(),
            balance: vesting_account
                .info
                .schedules
                .into_iter()
                .map(|vesting_schedule| vesting_schedule.start_point.amount)
                .sum(), // TODO: probably deduct vesting_account.info.released_amount
        })
        .collect::<Vec<_>>();

    to_binary(&ListBondersResponse { bonders })
}

pub fn query_bonding_status(
    _deps: Deps,
    env: Env,
    height: Option<u64>,
    _address: String,
) -> StdResult<BondingStatusResponse> {
    let height = height.unwrap_or(env.block.height);
    Ok(BondingStatusResponse {
        unbondable_abount: Uint128::zero(),
        bonding_enabled: false,
        height,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
