use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, FUND_COUNTER, PAUSED_UNTIL, PENDING_DISTRIBUTION, SHARES};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Storage, Uint128,
};
use cw2::set_contract_version;
use exec_control::pause::{
    can_pause, can_unpause, validate_duration, PauseError, PauseInfoResponse,
};

pub(crate) const CONTRACT_NAME: &str = "crates.io:distribution";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    let config = Config {
        denom: msg.denom,
        main_dao_address: deps.api.addr_validate(&msg.main_dao_address)?,
        security_dao_address: deps.api.addr_validate(&msg.security_dao_address)?,
    };
    CONFIG.save(deps.storage, &config)?;
    PAUSED_UNTIL.save(deps.storage, &None)?;
    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match get_pause_info(deps.as_ref(), &env)? {
        PauseInfoResponse::Paused { .. } => {
            return match msg {
                ExecuteMsg::Pause { duration } => execute_pause(deps, env, info.sender, duration),
                ExecuteMsg::Unpause {} => execute_unpause(deps, info.sender),
                _ => Err(ContractError::PauseError(PauseError::Paused {})),
            };
        }
        PauseInfoResponse::Unpaused {} => (),
    }

    match msg {
        ExecuteMsg::Pause { duration } => execute_pause(deps, env, info.sender, duration),
        ExecuteMsg::Unpause {} => execute_unpause(deps, info.sender),
        // permissioned - owner
        ExecuteMsg::TransferOwnership(new_owner) => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;
            execute_transfer_ownership(deps, info, new_owner_addr)
        }

        // permissioned - owner
        ExecuteMsg::SetShares { shares } => execute_set_shares(deps, info, shares),

        // permissionless
        ExecuteMsg::Fund {} => execute_fund(deps, info),

        // permissioned - owner of the share
        ExecuteMsg::Claim {} => execute_claim(deps, info),
    }
}

pub fn execute_pause(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    duration: u64,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    can_pause(
        &sender,
        &config.main_dao_address,
        &config.security_dao_address,
    )?;
    validate_duration(duration)?;

    let paused_until_height: u64 = env.block.height + duration;

    let already_paused_until = PAUSED_UNTIL.load(deps.storage)?;
    if already_paused_until.unwrap_or(0u64) >= paused_until_height {
        return Err(ContractError::PauseError(PauseError::InvalidDuration(
            "contracts are already paused for a greater or equal duration".to_string(),
        )));
    }

    PAUSED_UNTIL.save(deps.storage, &Some(paused_until_height))?;

    Ok(Response::new()
        .add_attribute("action", "execute_pause")
        .add_attribute("sender", sender)
        .add_attribute("paused_until_height", paused_until_height.to_string()))
}

pub fn execute_unpause(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    can_unpause(&sender, &config.main_dao_address)?;

    PAUSED_UNTIL.save(deps.storage, &None)?;

    Ok(Response::new()
        .add_attribute("action", "execute_unpause")
        .add_attribute("sender", sender))
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner_addr: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sender_addr = info.sender;
    let old_owner = config.main_dao_address;
    if sender_addr != old_owner {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.main_dao_address = new_owner_addr.clone();
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

pub fn execute_fund(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    let fund_counter = FUND_COUNTER.may_load(deps.storage)?.unwrap_or(0);
    let funds = get_denom_amount(info.funds, denom).unwrap_or(Uint128::zero());
    if funds.is_zero() {
        return Err(ContractError::NoFundsSent {});
    }
    let shares = SHARES
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    if shares.is_empty() {
        return Err(ContractError::NoSharesSent {});
    }
    let total_shares = shares
        .iter()
        .try_fold(Uint128::zero(), |acc, (_, s)| acc.checked_add(*s))?;
    if total_shares.is_zero() {
        return Err(ContractError::NoSharesSent {});
    }

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
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.main_dao_address {
        return Err(ContractError::Unauthorized {});
    }
    let mut new_shares = Vec::with_capacity(shares.len());
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

pub fn execute_claim(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let denom = config.denom;
    let sender = info.sender;
    let pending = PENDING_DISTRIBUTION
        .may_load(deps.storage, sender.clone())?
        .unwrap_or(Uint128::zero());
    if pending.is_zero() {
        return Err(ContractError::NoPendingDistribution {});
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Pending {} => to_json_binary(&query_pending(deps)?),
        QueryMsg::Shares {} => to_json_binary(&query_shares(deps)?),
        QueryMsg::PauseInfo {} => query_paused(deps, env),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_shares(deps: Deps) -> StdResult<Vec<(Addr, Uint128)>> {
    let shares = SHARES
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?;
    Ok(shares)
}

pub fn query_pending(deps: Deps) -> StdResult<Vec<(Addr, Uint128)>> {
    let pending = PENDING_DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?;
    Ok(pending)
}

pub fn query_paused(deps: Deps, env: Env) -> StdResult<Binary> {
    to_json_binary(&get_pause_info(deps, &env)?)
}

fn get_pause_info(deps: Deps, env: &Env) -> StdResult<PauseInfoResponse> {
    Ok(match PAUSED_UNTIL.may_load(deps.storage)?.unwrap_or(None) {
        Some(paused_until_height) => {
            if env.block.height.ge(&paused_until_height) {
                PauseInfoResponse::Unpaused {}
            } else {
                PauseInfoResponse::Paused {
                    until_height: paused_until_height,
                }
            }
        }
        None => PauseInfoResponse::Unpaused {},
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
