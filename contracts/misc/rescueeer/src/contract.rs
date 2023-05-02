#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, CosmosMsg, WasmMsg};
use neutron_bindings::bindings::msg::NeutronMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{Config, CONFIG};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: deps.api.addr_validate(msg.owner.as_str())?,
        true_admin: deps.api.addr_validate(msg.true_admin.as_str())?,
        eol: msg.eol,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::Execute { msgs } => execute_execute(deps, info, msgs),
        ExecuteMsg::TransferAdmin { address } => execute_transfer_admin(deps, env, info, address),
    }
}

pub fn execute_execute(
    deps: DepsMut,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<NeutronMsg>>,
) -> Result<Response<NeutronMsg>, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::default().add_messages(msgs))
}

pub fn execute_transfer_admin(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    addresses: String,
) -> Result<Response<NeutronMsg>, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if env.block.time.seconds() < config.eol {
        return Err(ContractError::TooSoon {});
    }
    let addr = deps.api.addr_validate(addresses.as_str())?;

    Ok(Response::default().add_message(
        CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
            contract_addr: addr.to_string(),
            admin: config.true_admin.to_string(),
        })))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
