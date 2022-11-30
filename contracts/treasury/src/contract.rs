#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Storage, Uint128,
};
use cw_storage_plus::KeyDeserialize;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StatsResponse};
use crate::state::{
    Config, BANK_BALANCE, CONFIG, DISTRIBUTION_BALANCE, LAST_BALANCE, PENDING_DISTRIBUTION, SHARES,
    TOTAL_BANK_SPENT, TOTAL_DISTRIBUTED, TOTAL_RECEIVED,
};

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
        min_time_elapsed_between_fundings: msg.min_time_elapsed_between_fundings,
        distribution_rate: msg.distribution_rate,
        owner: deps.api.addr_validate(&msg.owner)?,
        dao: deps.api.addr_validate(&msg.dao)?,
    };
    CONFIG.save(deps.storage, &config)?;
    TOTAL_RECEIVED.save(deps.storage, &Uint128::zero())?;
    TOTAL_BANK_SPENT.save(deps.storage, &Uint128::zero())?;
    TOTAL_DISTRIBUTED.save(deps.storage, &Uint128::zero())?;
    LAST_BALANCE.save(deps.storage, &Uint128::zero())?;
    DISTRIBUTION_BALANCE.save(deps.storage, &Uint128::zero())?;
    BANK_BALANCE.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let api = deps.api;
    match msg {
        // permissioned - owner
        ExecuteMsg::TransferOwnership(new_owner) => {
            exec_transfer_ownership(deps, info.sender, api.addr_validate(&new_owner)?)
        }
        // permissioned - dao
        ExecuteMsg::SetShares { shares } => exec_set_shares(deps, info, shares),
        // permissionless
        ExecuteMsg::Distribute {} => exec_distribute(deps, env),
        // permissionless
        ExecuteMsg::Grab {} => exec_grab(deps, env),
        // permissioned - dao
        ExecuteMsg::Payout { amount, recipient } => exec_payout(deps, info, env, amount, recipient),
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
        .add_attribute("action", "neutron/treasury/transfer_ownership")
        .add_attribute("previous_owner", old_owner)
        .add_attribute("new_owner", new_owner_addr))
}

pub fn exec_grab(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if config.distribution_rate == 0 {
        return Err(StdError::generic_err("distribution rate is zero"));
    }
    let last_balance = LAST_BALANCE.load(deps.storage)?;
    let current_balance = deps
        .querier
        .query_balance(env.contract.address, config.denom)?;
    if current_balance.amount.eq(&last_balance) {
        return Err(StdError::generic_err("no new funds to grab"));
    }
    let to_distribute = current_balance.amount.checked_sub(last_balance)?;
    let mut to_bank = to_distribute
        .checked_mul(config.distribution_rate.into())?
        .checked_div(100u128.into())?;
    let to_distribution = to_distribute.checked_sub(to_bank)?;
    // update bank
    let bank_balance = BANK_BALANCE.load(deps.storage)?;
    BANK_BALANCE.save(deps.storage, &(bank_balance.checked_add(to_bank)?))?;
    // update total received
    let total_received = TOTAL_RECEIVED.load(deps.storage)?;
    TOTAL_RECEIVED.save(deps.storage, &(total_received.checked_add(to_distribute)?))?;

    // // distribute to shares
    let shares = SHARES
        .range(deps.storage, None, None, Order::Ascending)
        .into_iter()
        .collect::<StdResult<Vec<_>>>()?;
    let sum_of_shares: Uint128 = shares.iter().fold(Uint128::zero(), |acc, (_, v)| acc + *v);
    let mut distributed = Uint128::zero();
    for (addr, share) in shares {
        let amount = to_distribution
            .checked_mul(share)?
            .checked_div(sum_of_shares)?;
        let p = PENDING_DISTRIBUTION.load(deps.storage, &addr);
        match p {
            Ok(p) => {
                PENDING_DISTRIBUTION.save(deps.storage, &addr, &(p.checked_add(amount)?))?;
            }
            Err(_) => {
                PENDING_DISTRIBUTION.save(deps.storage, &addr, &amount)?;
            }
        }
        distributed = distributed.checked_add(amount)?;
    }

    if distributed != to_distribution {
        to_bank = to_bank.checked_add(to_distribution.checked_sub(distributed)?)?;
    }

    // update bank
    let bank_balance = BANK_BALANCE.load(deps.storage)?;
    BANK_BALANCE.save(deps.storage, &(bank_balance.checked_add(to_bank)?))?;

    // update distribution balance
    let distribution_balance = DISTRIBUTION_BALANCE.load(deps.storage)?;
    DISTRIBUTION_BALANCE.save(
        deps.storage,
        &(distribution_balance.checked_add(distributed)?),
    )?;

    LAST_BALANCE.save(deps.storage, &current_balance.amount)?;

    Ok(Response::default()
        .add_attribute("action", "neutron/treasury/grab")
        .add_attribute("bank_balance", bank_balance)
        .add_attribute("distribution_balance", distribution_balance))
}

pub fn exec_payout(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    amount: Uint128,
    recipient: String,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    if info.sender != config.dao {
        return Err(StdError::generic_err("only dao can payout"));
    }
    let bank_balance = BANK_BALANCE.load(deps.storage)?;
    let distribute_balance = DISTRIBUTION_BALANCE.load(deps.storage)?;
    if amount > bank_balance {
        return Err(StdError::generic_err("insufficient funds"));
    }
    let current_balance = deps
        .querier
        .query_balance(env.contract.address, denom.clone())?;
    if bank_balance.checked_add(distribute_balance)? != current_balance.amount {
        return Err(StdError::generic_err("inconsistent state"));
    }
    BANK_BALANCE.save(deps.storage, &(bank_balance.checked_sub(amount)?))?;
    let total_bank_spent = TOTAL_BANK_SPENT.load(deps.storage)?;
    TOTAL_BANK_SPENT.save(deps.storage, &(total_bank_spent.checked_add(amount)?))?;
    LAST_BALANCE.save(deps.storage, &current_balance.amount.checked_sub(amount)?)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.clone(),
            amount: vec![Coin { denom, amount }],
        }))
        .add_attribute("action", "neutron/treasury/payout")
        .add_attribute("amount", amount)
        .add_attribute("recipient", recipient))
}

pub fn exec_distribute(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom.as_str();
    let distribution_balance = DISTRIBUTION_BALANCE.load(deps.storage)?;
    let bank_balance = BANK_BALANCE.load(deps.storage)?;
    let current_balance = deps.querier.query_balance(env.contract.address, denom)?;
    if bank_balance.checked_add(distribution_balance)? != current_balance.amount {
        return Err(StdError::generic_err("inconsistent state"));
    }
    let pending_distribution = PENDING_DISTRIBUTION
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .into_iter()
        .collect::<StdResult<Vec<_>>>()?;
    let mut msgs = vec![];
    let mut spent = Uint128::zero();
    for one in pending_distribution {
        let (addr, amount) = one;
        spent = spent.checked_add(amount)?;
        let msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: Addr::from_slice(&addr.clone())?.to_string(),
            amount: vec![Coin {
                denom: denom.to_string(),
                amount,
            }],
        });
        msgs.push(msg);
        PENDING_DISTRIBUTION.remove(deps.storage, &addr);
    }

    LAST_BALANCE.save(deps.storage, &current_balance.amount.checked_sub(spent)?)?;
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.storage)?;
    TOTAL_DISTRIBUTED.save(deps.storage, &(&total_distributed.checked_add(spent)?))?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "neutron/treasury/distribute"))
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

fn remove_all_shares(storage: &mut dyn Storage) -> StdResult<()> {
    let keys = SHARES
        .keys(storage, None, None, Order::Ascending)
        .into_iter()
        .fold(vec![], |mut acc, key| {
            acc.push(key.unwrap());
            acc
        });

    for key in keys.iter() {
        SHARES.remove(storage, &key);
    }
    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Stats {} => to_binary(&query_stats(deps)?),
        QueryMsg::Shares {} => to_binary(&query_shares(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_stats(deps: Deps) -> StdResult<StatsResponse> {
    let total_received = TOTAL_RECEIVED.load(deps.storage)?;
    let total_bank_spent = TOTAL_BANK_SPENT.load(deps.storage)?;
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.storage)?;
    let last_balance = LAST_BALANCE.load(deps.storage)?;
    let distribution_balance = DISTRIBUTION_BALANCE.load(deps.storage)?;
    let bank_balance = BANK_BALANCE.load(deps.storage)?;

    Ok(StatsResponse {
        total_received,
        total_bank_spent,
        total_distributed,
        last_balance,
        distribution_balance,
        bank_balance,
    })
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
