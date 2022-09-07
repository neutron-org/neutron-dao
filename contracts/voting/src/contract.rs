use cosmwasm_std::{
    Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, to_binary, Uint128,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, VotingPowerResponse,
};
use crate::msg::QueryMsg::VotingPowers;
use crate::state::{OWNER, TOKENS_LOCKED};

const CONTRACT_NAME: &str = "crates.io:neutron-dao";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//--------------------------------------------------------------------------------------------------
// Instantiation
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    OWNER.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;

    init_voting(deps)?;

    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env:Env,  info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let api = deps.api;
    match msg {
        ExecuteMsg::TransferOwnership(new_owner) => {
            transfer_ownership(deps, info.sender, api.addr_validate(&new_owner)?)
        }
        ExecuteMsg::InitVoting() => init_voting(deps)
    }
}

pub fn transfer_ownership(
    deps: DepsMut,
    sender_addr: Addr,
    new_owner_addr: Addr,
) -> StdResult<Response> {
    let owner_addr = OWNER.load(deps.storage)?;
    if sender_addr != owner_addr {
        return Err(StdError::generic_err("only owner can transfer ownership"));
    }

    OWNER.save(deps.storage, &new_owner_addr)?;

    Ok(Response::new()
        .add_attribute("action", "neutron/voting/transfer_ownership")
        .add_attribute("previous_owner", owner_addr)
        .add_attribute("new_owner", new_owner_addr))
}

pub fn init_voting(deps: DepsMut) -> StdResult<Response> {
    let value1 = Uint128::new(1000);
    let value2 = Uint128::new(300);
    TOKENS_LOCKED.save(deps.storage, &deps.api.addr_validate("neutron1mjk79fjjgpplak5wq838w0yd982gzkyf8fxu8u")?, &value1)?;
    TOKENS_LOCKED.save(deps.storage, &deps.api.addr_validate("neutron17dtl0mjt3t77kpuhg2edqzjpszulwhgzcdvagh")?, &value2)?;
    Ok(Response::default())
}
//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env:Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::VotingPower {
            user,
        } => to_binary(&query_voting_power(deps, api.addr_validate(&user)?)?),
        VotingPowers {} => to_binary(&query_voting_powers(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        owner: OWNER.load(deps.storage)?.into(),
    })
}

pub fn query_voting_power(deps: Deps, user_addr: Addr) -> StdResult<VotingPowerResponse> {
    let voting_power = match TOKENS_LOCKED.may_load(deps.storage, &user_addr) {
        Ok(Some(voting_power)) => voting_power,
        Ok(None) => Uint128::zero(),
        Err(err) => return Err(err),
    };

    Ok(VotingPowerResponse {
        user: user_addr.to_string(),
        voting_power,
    })
}

pub fn query_voting_powers(
    deps: Deps,
) -> StdResult<Vec<VotingPowerResponse>> {
    let voting_powers = TOKENS_LOCKED.range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|res| {
            let (addr, voting_power) = res?;
            Ok(VotingPowerResponse { user: addr.to_string(), voting_power })
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(voting_powers)
}

