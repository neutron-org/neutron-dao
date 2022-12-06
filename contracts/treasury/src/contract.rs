#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg,
};

use crate::msg::{DistributionMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StatsResponse};
use crate::state::{
    Config, BANK_BALANCE, CONFIG, LAST_BALANCE, LAST_GRAB_TIME, TOTAL_BANK_SPENT,
    TOTAL_DISTRIBUTED, TOTAL_RECEIVED,
};

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
        min_period: msg.min_period,
        distribution_contract: deps.api.addr_validate(msg.distribution_contract.as_str())?,
        distribution_rate: msg.distribution_rate,
        owner: deps.api.addr_validate(&msg.owner)?,
        dao: deps.api.addr_validate(&msg.dao)?,
    };
    CONFIG.save(deps.storage, &config)?;
    TOTAL_RECEIVED.save(deps.storage, &Uint128::zero())?;
    TOTAL_BANK_SPENT.save(deps.storage, &Uint128::zero())?;
    TOTAL_DISTRIBUTED.save(deps.storage, &Uint128::zero())?;
    LAST_GRAB_TIME.save(deps.storage, &0)?;
    LAST_BALANCE.save(deps.storage, &Uint128::zero())?;
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
        // permissionless
        ExecuteMsg::Distribute {} => exec_distribute(deps, env),
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

pub fn exec_distribute(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    let current_time = env.block.time.seconds();
    if current_time - LAST_GRAB_TIME.load(deps.storage)? < config.min_period {
        return Err(StdError::generic_err("too soon to collect"));
    }
    LAST_GRAB_TIME.save(deps.storage, &current_time)?;
    // TODO: do we need it?
    // if config.distribution_rate == 0 {
    //     return Err(StdError::generic_err("distribution rate is zero"));
    // }
    let last_balance = LAST_BALANCE.load(deps.storage)?;
    let current_balance = deps
        .querier
        .query_balance(env.contract.address, denom.clone())?;
    if current_balance.amount.eq(&last_balance) {
        return Err(StdError::generic_err("no new funds to grab"));
    }
    let balance_delta = current_balance.amount.checked_sub(last_balance)?;
    let to_distribution = balance_delta
        .checked_mul(config.distribution_rate.into())?
        .checked_div(100u128.into())?;
    let to_bank = balance_delta.checked_sub(to_distribution)?;
    // update total received
    let total_received = TOTAL_RECEIVED.load(deps.storage)?;
    TOTAL_RECEIVED.save(deps.storage, &(total_received.checked_add(balance_delta)?))?;
    // update bank
    let bank_balance = BANK_BALANCE.load(deps.storage)?;
    BANK_BALANCE.save(deps.storage, &(bank_balance.checked_add(to_bank)?))?;
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.storage)?;
    TOTAL_DISTRIBUTED.save(
        deps.storage,
        &(total_distributed.checked_add(to_distribution)?),
    )?;

    LAST_BALANCE.save(
        deps.storage,
        &current_balance.amount.checked_sub(to_distribution)?,
    )?;
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.distribution_contract.to_string(),
        funds: coins(to_distribution.u128(), denom),
        msg: to_binary(&DistributionMsg::Fund {})?,
    });

    Ok(Response::default()
        .add_message(msg)
        .add_attribute("action", "neutron/treasury/grab")
        .add_attribute("bank_balance", bank_balance)
        .add_attribute("distributed", to_distribution))
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
    if amount.gt(&bank_balance) {
        return Err(StdError::generic_err("insufficient funds"));
    }
    let current_balance = deps
        .querier
        .query_balance(env.contract.address, denom.clone())?;
    if bank_balance.gt(&current_balance.amount) {
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

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Stats {} => to_binary(&query_stats(deps)?),
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
    let bank_balance = BANK_BALANCE.load(deps.storage)?;

    Ok(StatsResponse {
        total_received,
        total_bank_spent,
        total_distributed,
        last_balance,
        bank_balance,
    })
}
