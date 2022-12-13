#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Storage, Uint128,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, FUND_COUNTER, PENDING_DISTRIBUTION, SHARES};

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
            execute_transfer_ownership(deps, info, api.addr_validate(&new_owner)?)
        }

        // permissioned - owner
        ExecuteMsg::SetShares { shares } => execute_set_shares(deps, info, shares),

        // permissionless
        ExecuteMsg::Fund {} => execute_fund(deps, info),

        // permissioned - owner of the share
        ExecuteMsg::Claim {} => execute_claim(deps, info),
    }
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner_addr: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let old_owner = config.owner;
    let sender_addr = info.sender;
    if sender_addr != old_owner {
        return Err(StdError::generic_err("unauthorized"));
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

pub fn execute_fund(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    let fund_counter = FUND_COUNTER.may_load(deps.storage)?.unwrap_or(0);
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
    let total_shares = shares.iter().fold(Uint128::zero(), |acc, (_, s)| acc + s);
    let mut spent = Uint128::zero();
    let mut resp = Response::new().add_attribute("action", "neutron/distribution/fund");

    for (addr, share) in shares.iter() {
        let amount = funds.checked_mul(*share)?.checked_div(total_shares)?;
        let pending = PENDING_DISTRIBUTION
            .may_load(deps.storage, addr.clone())?
            .unwrap_or(Uint128::zero());
        PENDING_DISTRIBUTION.save(deps.storage, addr.clone(), &(pending.checked_add(amount)?))?;
        spent = spent.checked_add(amount)?;
        resp = resp
            .add_attribute("address", addr)
            .add_attribute("amount", amount);
    }
    let remaining = funds.checked_sub(spent)?;
    if !remaining.is_zero() {
        let index = fund_counter % shares.len() as u64;
        let key = &shares.get(index as usize).unwrap().0;
        let pending = PENDING_DISTRIBUTION
            .may_load(deps.storage, key.clone())?
            .unwrap_or(Uint128::zero());
        PENDING_DISTRIBUTION.save(
            deps.storage,
            key.clone(),
            &(pending.checked_add(remaining)?),
        )?;
        resp = resp
            .add_attribute("remainder_address", key)
            .add_attribute("remainder_amount", remaining);
    }
    FUND_COUNTER.save(deps.storage, &(fund_counter + 1))?;
    Ok(resp)
}

pub fn execute_set_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares: Vec<(String, Uint128)>,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }
    let mut new_shares = vec![];
    for (addr, share) in shares {
        let addr = deps.api.addr_validate(&addr)?;
        new_shares.push((addr, share));
    }
    remove_all_shares(deps.storage)?;
    for (addr, shares) in new_shares.iter() {
        SHARES.save(deps.storage, addr.clone(), shares)?;
    }
    Ok(Response::new()
        .add_attribute("action", "neutron/distribution/set_shares")
        .add_attribute("shares", format!("{:?}", new_shares)))
}

pub fn remove_all_shares(storage: &mut dyn Storage) -> StdResult<()> {
    let shares = SHARES
        .keys(storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    for addr in shares {
        SHARES.remove(storage, addr);
    }
    Ok(())
}

pub fn execute_claim(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    let sender = info.sender;
    let pending = PENDING_DISTRIBUTION
        .may_load(deps.storage, sender.clone())?
        .unwrap_or(Uint128::zero());
    if pending.is_zero() {
        return Err(StdError::generic_err("no pending distribution"));
    }
    PENDING_DISTRIBUTION.remove(deps.storage, sender.clone());
    Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: sender.to_string(),
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
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    let mut res: Vec<(String, Uint128)> = vec![];
    for (addr, shares) in shares {
        res.push((addr.to_string(), shares));
    }
    Ok(res)
}

pub fn query_pending(deps: Deps) -> StdResult<Vec<(String, Uint128)>> {
    let pending = PENDING_DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    let mut res: Vec<(String, Uint128)> = vec![];
    for (addr, pending) in pending {
        res.push((addr.to_string(), pending));
    }
    Ok(res)
}
