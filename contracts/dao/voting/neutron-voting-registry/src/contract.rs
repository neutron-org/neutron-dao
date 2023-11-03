use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, VotingVault};
use crate::state::{Config, VotingVaultState, CONFIG, DAO, VAULT_STATES};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Item;
use cwd_interface::voting::{self, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse};
use neutron_vault::msg::QueryMsg as VaultQueryMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub(crate) const CONTRACT_NAME: &str = "crates.io:neutron-voting-registry";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    for vault in msg.voting_vaults.iter() {
        VAULT_STATES.save(
            deps.storage,
            deps.api.addr_validate(vault)?,
            &VotingVaultState::Active,
            env.block.height,
        )?
    }

    let config = Config { owner };

    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", config.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddVotingVault {
            new_voting_vault_contract,
        } => execute_add_voting_vault(deps, env, info, new_voting_vault_contract),
        ExecuteMsg::DeactivateVotingVault {
            voting_vault_contract,
        } => execute_deactivate_voting_vault(deps, env, info, voting_vault_contract),
        ExecuteMsg::ActivateVotingVault {
            voting_vault_contract,
        } => execute_activate_voting_vault(deps, env, info, voting_vault_contract),
        ExecuteMsg::UpdateConfig { owner } => execute_update_config(deps, info, owner),
    }
}

pub fn execute_add_voting_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_voting_vault_contract: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let voting_vault_contract_addr = deps.api.addr_validate(&new_voting_vault_contract)?;
    if VAULT_STATES
        .load(deps.storage, voting_vault_contract_addr.clone())
        .is_ok()
    {
        return Err(ContractError::VotingVaultAlreadyExists {});
    }
    VAULT_STATES.save(
        deps.storage,
        voting_vault_contract_addr,
        &VotingVaultState::Active,
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "add_voting_vault")
        .add_attribute("vault", new_voting_vault_contract))
}

pub fn execute_deactivate_voting_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    voting_vault_contract: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let voting_vault_contract_addr = deps.api.addr_validate(&voting_vault_contract)?;

    let vault_state = VAULT_STATES.load(deps.storage, voting_vault_contract_addr.clone())?;
    if vault_state == VotingVaultState::Inactive {
        return Err(ContractError::VotingVaultAlreadyInactive {});
    }

    VAULT_STATES.save(
        deps.storage,
        voting_vault_contract_addr,
        &VotingVaultState::Inactive,
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "deactivate_voting_vault")
        .add_attribute("vault", voting_vault_contract))
}

pub fn execute_activate_voting_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    voting_vault_contract: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let voting_vault_contract_addr = deps.api.addr_validate(&voting_vault_contract)?;

    let vault_state = VAULT_STATES.load(deps.storage, voting_vault_contract_addr.clone())?;
    if vault_state == VotingVaultState::Active {
        return Err(ContractError::VotingVaultAlreadyActive {});
    }

    VAULT_STATES.save(
        deps.storage,
        voting_vault_contract_addr,
        &VotingVaultState::Active,
        env.block.height,
    )?;

    Ok(Response::new()
        .add_attribute("action", "activate_voting_vault")
        .add_attribute("vault", voting_vault_contract))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner = deps.api.addr_validate(&new_owner)?;

    config.owner = new_owner;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VotingPowerAtHeight { address, height } => {
            to_json_binary(&query_voting_power_at_height(deps, env, address, height)?)
        }
        QueryMsg::TotalPowerAtHeight { height } => {
            to_json_binary(&query_total_power_at_height(deps, env, height)?)
        }
        QueryMsg::Info {} => query_info(deps),
        QueryMsg::Dao {} => query_dao(deps),
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::VotingVaults { height } => {
            to_json_binary(&query_voting_vaults(deps, env, height)?)
        }
    }
}

pub fn query_voting_vaults(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<Vec<VotingVault>> {
    let vaults = VAULT_STATES
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<Addr>, StdError>>()?;
    let height = height.unwrap_or(env.block.height);

    let mut voting_vaults: Vec<VotingVault> = vec![];
    for vault in vaults {
        if let Some(state) = VAULT_STATES.may_load_at_height(deps.storage, vault.clone(), height)? {
            let description: String = deps
                .querier
                .query_wasm_smart(vault.clone(), &VaultQueryMsg::Description {})?;
            let name: String = deps
                .querier
                .query_wasm_smart(vault.clone(), &VaultQueryMsg::Name {})?;

            voting_vaults.push(VotingVault {
                address: vault.to_string(),
                name,
                description,
                state,
            })
        }
    }

    Ok(voting_vaults)
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    let vaults = VAULT_STATES
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<Addr>, StdError>>()?;
    let height = height.unwrap_or(env.block.height);

    let mut resp = VotingPowerAtHeightResponse {
        power: Default::default(),
        height,
    };
    for vault in vaults {
        if let Some(vault_state) =
            VAULT_STATES.may_load_at_height(deps.storage, vault.clone(), height)?
        {
            if vault_state == VotingVaultState::Active {
                let vp_in_vault: VotingPowerAtHeightResponse = deps.querier.query_wasm_smart(
                    vault,
                    &voting::Query::VotingPowerAtHeight {
                        height: Some(height),
                        address: address.clone(),
                    },
                )?;

                resp.power = resp.power.checked_add(vp_in_vault.power)?;
            }
        }
    }

    Ok(resp)
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let vaults = VAULT_STATES
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<Addr>, StdError>>()?;
    let height = height.unwrap_or(env.block.height);

    let mut resp = TotalPowerAtHeightResponse {
        power: Default::default(),
        height,
    };
    for vault in vaults {
        if let Some(vault_state) =
            VAULT_STATES.may_load_at_height(deps.storage, vault.clone(), height)?
        {
            if vault_state == VotingVaultState::Active {
                let vp_in_vault: TotalPowerAtHeightResponse = deps.querier.query_wasm_smart(
                    vault,
                    &voting::Query::TotalPowerAtHeight {
                        height: Some(height),
                    },
                )?;

                resp.power = resp.power.checked_add(vp_in_vault.power)?;
            }
        }
    }

    Ok(resp)
}

pub fn query_info(deps: Deps) -> StdResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    to_json_binary(&voting::InfoResponse { info })
}

pub fn query_dao(deps: Deps) -> StdResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    to_json_binary(&dao)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // read the prev version config
    #[derive(Serialize, Deserialize, JsonSchema)]
    struct OldConfig {
        pub owner: Addr,
        pub voting_vaults: Vec<Addr>,
    }
    let old_config: OldConfig = Item::new("config").load(deps.storage)?;

    // move vaults from old config to a dedicated Item
    for vault in old_config.voting_vaults {
        VAULT_STATES.save(deps.storage, vault, &VotingVaultState::Active, 1u64)?;
    }

    // overwrite value behind config key
    CONFIG.save(
        deps.storage,
        &Config {
            owner: old_config.owner,
        },
    )?;

    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
