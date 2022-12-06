#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Storage, Uint128,
};
use cw_storage_plus::KeyDeserialize;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, PENDING_DISTRIBUTION, SHARES};

// const CONTRACT_NAME: &str = "crates.io:neutron-treasury";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    let config = Config {
        denom: msg.denom,
        owner: deps.api.addr_validate(&msg.owner)?,
        dao: deps.api.addr_validate(&msg.dao)?,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let api = deps.api;
    match msg {
        // permissioned - owner
        ExecuteMsg::TransferOwnership(new_owner) => {
            exec_transfer_ownership(deps, info.sender, api.addr_validate(&new_owner)?)
        }

        // permissioned - dao
        ExecuteMsg::SetShares { shares } => exec_set_shares(deps, info, shares),

        // permissionless
        ExecuteMsg::Fund {} => exec_fund(deps, info),

        // permissioned - owner of the share
        ExecuteMsg::Claim {} => exec_claim(deps, info),
    }
}

pub fn exec_transfer_ownership(
    deps: DepsMut,
    sender_addr: Addr,
    new_owner_addr: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let old_owner = config.owner;
    if sender_addr != old_owner {
        return Err(StdError::generic_err("only owner can transfer ownership"));
    }

    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner = new_owner_addr.clone();
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "neutron/distribution/transfer_ownership")
        .add_attribute("previous_owner", old_owner)
        .add_attribute("new_owner", new_owner_addr))
}

fn get_denom_amount(coins: Vec<Coin>, denom: String) -> Option<Uint128> {
    coins
        .into_iter()
        .find(|c| c.denom == denom)
        .map(|c| c.amount)
}

pub fn exec_fund(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    let funds = get_denom_amount(info.funds, denom).unwrap_or(Uint128::zero());
    if funds.is_zero() {
        return Err(StdError::generic_err("no funds sent"));
    }
    let shares = SHARES
        .range(deps.storage, None, None, Order::Ascending)
        .into_iter()
        .collect::<StdResult<Vec<_>>>()?;
    if shares.is_empty() {
        return Err(StdError::generic_err("no shares set"));
    }
    let mut spent = Uint128::zero();
    let total_shares = shares
        .clone()
        .into_iter()
        .fold(Uint128::zero(), |acc, (_, s)| acc + s);
    for (addr, share) in shares {
        let amount = funds.checked_mul(share)?.checked_div(total_shares)?;
        spent += amount;
        let pending = PENDING_DISTRIBUTION
            .may_load(deps.storage, &addr)?
            .unwrap_or(Uint128::zero());
        PENDING_DISTRIBUTION.save(deps.storage, &addr, &(pending.checked_add(amount)?))?;
    }
    Ok(Response::new().add_attribute("action", "neutron/distribution/fund"))
}

pub fn exec_set_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares: Vec<(String, Uint128)>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.dao {
        return Err(StdError::generic_err("only dao can set shares"));
    }
    let mut new_shares = vec![];
    for (addr, share) in shares {
        let addr = deps.api.addr_validate(&addr)?;
        let addr_raw = addr.as_bytes();
        SHARES.save(deps.storage, addr_raw, &share)?;
        new_shares.push((addr_raw.to_vec(), share));
    }
    remove_all_shares(deps.storage)?;
    for (addr, shares) in new_shares.clone() {
        SHARES.save(deps.storage, &addr, &shares)?;
    }
    Ok(Response::new()
        .add_attribute("action", "neutron/treasury/set_shares")
        .add_attribute("shares", format!("{:?}", new_shares)))
}

pub fn remove_all_shares(storage: &mut dyn Storage) -> StdResult<()> {
    let shares = SHARES
        .range(storage, None, None, Order::Ascending)
        .into_iter()
        .collect::<StdResult<Vec<_>>>()?;
    for (addr, _) in shares {
        SHARES.remove(storage, &addr);
    }
    Ok(())
}

pub fn exec_claim(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    let sender = info.sender.as_bytes();
    let pending = PENDING_DISTRIBUTION
        .may_load(deps.storage, sender)?
        .unwrap_or(Uint128::zero());
    if pending.is_zero() {
        return Err(StdError::generic_err("no pending distribution"));
    }
    PENDING_DISTRIBUTION.remove(deps.storage, sender);
    Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom,
            amount: pending,
        }],
    })))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Pending {} => to_binary(&query_pending(deps)?),
        QueryMsg::Shares {} => to_binary(&query_shares(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_shares(deps: Deps) -> StdResult<Vec<(String, Uint128)>> {
    let shares = SHARES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .into_iter()
        .collect::<StdResult<Vec<_>>>()?;
    let mut res: Vec<(String, Uint128)> = vec![];
    for (addr, shares) in shares.iter() {
        res.push((Addr::from_slice(addr)?.to_string(), *shares));
    }
    Ok(res)
}

pub fn query_pending(deps: Deps) -> StdResult<Vec<(String, Uint128)>> {
    let pending = PENDING_DISTRIBUTION
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .into_iter()
        .collect::<StdResult<Vec<_>>>()?;
    let mut res: Vec<(String, Uint128)> = vec![];
    for (addr, pending) in pending.iter() {
        res.push((Addr::from_slice(addr)?.to_string(), *pending));
    }
    Ok(res)
}
