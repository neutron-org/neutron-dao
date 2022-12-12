#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg,
};

use crate::msg::{DistributeMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StatsResponse};
use crate::state::{
    Config, BANK_BALANCE, CONFIG, LAST_BALANCE, LAST_DISTRIBUTION_TIME, TOTAL_BANK_SPENT,
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
    };
    CONFIG.save(deps.storage, &config)?;
    TOTAL_RECEIVED.save(deps.storage, &Uint128::zero())?;
    TOTAL_BANK_SPENT.save(deps.storage, &Uint128::zero())?;
    TOTAL_DISTRIBUTED.save(deps.storage, &Uint128::zero())?;
    LAST_DISTRIBUTION_TIME.save(deps.storage, &0)?;
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
            execute_transfer_ownership(deps, info.sender, api.addr_validate(&new_owner)?)
        }
        // permissionless
        ExecuteMsg::Distribute {} => execute_distribute(deps, env),
        // permissioned - owner
        ExecuteMsg::Payout { amount, recipient } => execute_payout(deps, info, env, amount, recipient),
        // permissioned - owner
        ExecuteMsg::UpdateConfig {
            distribution_rate,
            min_period,
            distribution_contract,
        } => execute_update_config(
            deps,
            info,
            distribution_rate,
            min_period,
            distribution_contract,
        ),
    }
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    sender_addr: Addr,
    new_owner_addr: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let old_owner = config.owner;
    if sender_addr != old_owner {
        return Err(StdError::generic_err("unauthorized"));
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

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    distribution_rate: Option<u8>,
    min_period: Option<u64>,
    distribution_contract: Option<String>,
) -> StdResult<Response> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(min_period) = min_period {
        config.min_period = min_period;
    }
    if let Some(distribution_contract) = distribution_contract {
        config.distribution_contract = deps.api.addr_validate(distribution_contract.as_str())?;
    }
    if let Some(distribution_rate) = distribution_rate {
        config.distribution_rate = distribution_rate;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "neutron/treasury/update_config")
        .add_attribute("denom", config.denom)
        .add_attribute("min_period", config.min_period.to_string())
        .add_attribute("distribution_contract", config.distribution_contract)
        .add_attribute("distribution_rate", config.distribution_rate.to_string())
        .add_attribute("owner", config.owner))
}

pub fn execute_distribute(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    let current_time = env.block.time.seconds();
    if current_time - LAST_DISTRIBUTION_TIME.load(deps.storage)? < config.min_period {
        return Err(StdError::generic_err("too soon to distribute"));
    }
    LAST_DISTRIBUTION_TIME.save(deps.storage, &current_time)?;
    let last_balance = LAST_BALANCE.load(deps.storage)?;
    let current_balance = deps
        .querier
        .query_balance(env.contract.address, denom.clone())?;
    if current_balance.amount.eq(&last_balance) {
        return Err(StdError::generic_err("no new funds to distribute"));
    }
    let balance_delta = current_balance.amount.checked_sub(last_balance)?;
    let to_distribute = balance_delta
        .checked_mul(config.distribution_rate.into())?
        .checked_div(100u128.into())?;
    let to_bank = balance_delta.checked_sub(to_distribute)?;
    // update total received
    let total_received = TOTAL_RECEIVED.load(deps.storage)?;
    TOTAL_RECEIVED.save(deps.storage, &(total_received.checked_add(balance_delta)?))?;
    // update bank
    let bank_balance = BANK_BALANCE.load(deps.storage)?;
    BANK_BALANCE.save(deps.storage, &(bank_balance.checked_add(to_bank)?))?;
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.storage)?;
    TOTAL_DISTRIBUTED.save(
        deps.storage,
        &(total_distributed.checked_add(to_distribute)?),
    )?;

    LAST_BALANCE.save(
        deps.storage,
        &current_balance.amount.checked_sub(to_distribute)?,
    )?;
    let mut resp = Response::default();
    if !to_distribute.is_zero() {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.distribution_contract.to_string(),
            funds: coins(to_distribute.u128(), denom),
            msg: to_binary(&DistributeMsg::Fund {})?,
        });
        deps.api.debug(format!("WASMDEBUG: {:?}", msg).as_str());
        resp = resp.add_message(msg)
    }
    Ok(resp
        .add_attribute("action", "neutron/treasury/distribute")
        .add_attribute("bank_balance", bank_balance)
        .add_attribute("distributed", to_distribute))
}

pub fn execute_payout(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    amount: Uint128,
    recipient: String,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
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
