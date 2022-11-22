#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;
// use cw_controllers::ClaimsResponse;
use cwd_interface::voting::{TotalPowerAtHeightResponse, VotingPowerAtHeightResponse, };
use cwd_interface::{Admin, voting};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use crate::state::{Config, CONFIG, DAO, STAKED_TOTAL};

pub(crate) const CONTRACT_NAME: &str = "crates.io:cwd-voting-vault";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .as_ref()
        .map(|owner| match owner {
            Admin::Address { addr } => deps.api.addr_validate(addr),
            Admin::CoreModule {} => Ok(info.sender.clone()),
        })
        .transpose()?;
    let manager = msg
        .manager
        .map(|manager| deps.api.addr_validate(&manager))
        .transpose()?;

    let staking = deps.api.addr_validate(&msg.staking)?;

    let config = Config {
        owner,
        manager,
        staking,
    };

    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute(
            "owner",
            config
                .owner
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        )
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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddStakingContract { new_staking_contract} => execute_add_staking(deps, env, info, new_staking_contract),
        ExecuteMsg::UpdateConfig {
            owner,
            manager,
        } => execute_update_config(deps, info, owner, manager),
    }
}

pub fn execute_add_staking(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _new_staking_contact: String
) -> Result<Response, ContractError> {
    //TODO fill this
    // let config = CONFIG.load(deps.storage)?;

    Ok(Response::new())
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Option<String>,
    new_manager: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if Some(info.sender.clone()) != config.owner && Some(info.sender.clone()) != config.manager {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = new_owner
        .map(|new_owner| deps.api.addr_validate(&new_owner))
        .transpose()?;
    let new_manager = new_manager
        .map(|new_manager| deps.api.addr_validate(&new_manager))
        .transpose()?;

    if Some(info.sender) != config.owner && new_owner != config.owner {
        return Err(ContractError::OnlyOwnerCanChangeOwner {});
    };

    config.owner = new_owner;
    config.manager = new_manager;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute(
            "owner",
            config
                .owner
                .map(|a| a.to_string())
                .unwrap_or_else(|| "None".to_string()),
        )
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
        // QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),
        QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
        // QueryMsg::ListStakers { start_after, limit } => {
        //     query_list_stakers(deps, start_after, limit)
        // }
        QueryMsg::Staking {} => query_staking(deps)
    }
}

pub fn query_staking(
    deps: Deps,
) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    to_binary(&config.staking)
}

pub fn query_voting_power_at_height(
    deps: Deps,
    _env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<Binary> {
    let staking = CONFIG.load(deps.storage)?.staking;
    let total_power: VotingPowerAtHeightResponse = deps
        .querier
        .query_wasm_smart(staking, &voting::Query::VotingPowerAtHeight { height, address})?;
    to_binary(&total_power)
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let height = height.unwrap_or(env.block.height);
    let power = STAKED_TOTAL
        .may_load_at_height(deps.storage, height)?
        .unwrap_or_default();
    Ok(TotalPowerAtHeightResponse { power, height })
}

pub fn query_info(deps: Deps) -> StdResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    to_binary(&voting::InfoResponse { info })
}

pub fn query_dao(deps: Deps) -> StdResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    to_binary(&dao)
}

// pub fn query_claims(deps: Deps, address: String) -> StdResult<Binary> {
//     let staking = CONFIG.load(deps.storage)?.staking;
//     let claims: ClaimsResponse = deps
//         .querier
//         .query_wasm_smart(staking, &voting::Query::Claims { address })?;
//     to_binary(&claims)
// }
//
// pub fn query_list_stakers(
//     deps: Deps,
//     start_after: Option<String>,
//     limit: Option<u32>,
// ) -> StdResult<Binary> {
//     let staking = CONFIG.load(deps.storage)?.staking;
//     let stakers = deps
//         .querier
//         .query_wasm_smart(staking, &voting::Query::ListStakers { start_after, limit })?;
//     to_binary(&stakers)
//
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
