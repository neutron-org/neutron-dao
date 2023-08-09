use crate::contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{coins, Addr, Coin, Empty, Uint128};
use cw_multi_test::{custom_app, App, AppResponse, Contract, ContractWrapper, Executor};
use cwd_interface::voting::InfoResponse;
use neutron_lockdrop_vault_for_cl_pools::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use neutron_lockdrop_vault_for_cl_pools::types::Config;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DAO_ADDR: &str = "dao";
const NAME: &str = "name";
const NEW_NAME: &str = "new_name";
const DESCRIPTION: &str = "description";
const NEW_DESCRIPTION: &str = "new description";
const LOCKDROP_ADDR: &str = "lockdrop";
const ORACLE_USDC_ADDR: &str = "oracle_usdc";
const ORACLE_ATOM_ADDR: &str = "oracle_atom";
const NEW_LOCKDROP_ADDR: &str = "new_lockdrop";
const NEW_ORACLE_USDC_ADDR: &str = "new_oracle_usdc";
const NEW_ORACLE_ATOM_ADDR: &str = "new_oracle_atom";
const ADDR1: &str = "addr1";
const ADDR2: &str = "addr2";
const DENOM: &str = "ujuno";
const INIT_BALANCE: Uint128 = Uint128::new(10000);

fn vault_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn mock_app() -> App {
    custom_app(|r, _a, s| {
        r.bank
            .init_balance(
                s,
                &Addr::unchecked(DAO_ADDR),
                vec![Coin {
                    denom: DENOM.to_string(),
                    amount: INIT_BALANCE,
                }],
            )
            .unwrap();
        r.bank
            .init_balance(
                s,
                &Addr::unchecked(ADDR1),
                vec![Coin {
                    denom: DENOM.to_string(),
                    amount: INIT_BALANCE,
                }],
            )
            .unwrap();
        r.bank
            .init_balance(
                s,
                &Addr::unchecked(ADDR2),
                vec![Coin {
                    denom: DENOM.to_string(),
                    amount: INIT_BALANCE,
                }],
            )
            .unwrap();
    })
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct EmptyMsg {}

fn instantiate_vault(app: &mut App, id: u64, msg: InstantiateMsg) -> Addr {
    app.instantiate_contract(id, Addr::unchecked(DAO_ADDR), &msg, &[], "vault", None)
        .unwrap()
}

fn bond_tokens(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    amount: u128,
    denom: &str,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::Bond {},
        &coins(amount, denom),
    )
}

fn unbond_tokens(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    amount: u128,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::Unbond {
            amount: Uint128::new(amount),
        },
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
fn update_config(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    owner: Option<String>,
    lockdrop_contract: Option<String>,
    oracle_usdc_contract: Option<String>,
    oracle_atom_contract: Option<String>,
    name: Option<String>,
    description: Option<String>,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::UpdateConfig {
            owner,
            lockdrop_contract,
            oracle_usdc_contract,
            oracle_atom_contract,
            name,
            description,
        },
        &[],
    )
}

fn get_config(app: &mut App, contract_addr: Addr) -> Config {
    app.wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
        .unwrap()
}

fn get_dao(app: &App, contract_addr: &Addr) -> String {
    app.wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Dao {})
        .unwrap()
}

#[test]
fn test_instantiate() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );
    assert_eq!(get_dao(&app, &addr), String::from(DAO_ADDR));
}

#[test]
#[should_panic(expected = "Bonding is not available for this contract")]
fn test_bond() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    // Try and bond an invalid denom
    bond_tokens(&mut app, addr, ADDR1, 100, DENOM).unwrap();
}

#[test]
#[should_panic(expected = "Direct unbonding is not available for this contract")]
fn test_unbond() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    unbond_tokens(&mut app, addr, ADDR1, 100).unwrap();
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_update_config_unauthorized() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    // From ADDR2, so not owner
    update_config(
        &mut app,
        addr,
        ADDR2,
        Some(ADDR1.to_string()),
        Some(NEW_LOCKDROP_ADDR.to_string()),
        Some(NEW_ORACLE_USDC_ADDR.to_string()),
        Some(NEW_ORACLE_ATOM_ADDR.to_string()),
        Some(NEW_NAME.to_string()),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();
}

#[test]
fn test_update_config_as_owner() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    // Change owner, description, name and lockdrop contract
    update_config(
        &mut app,
        addr.clone(),
        DAO_ADDR,
        Some(ADDR1.to_string()),
        Some(NEW_LOCKDROP_ADDR.to_string()),
        Some(NEW_ORACLE_USDC_ADDR.to_string()),
        Some(NEW_ORACLE_ATOM_ADDR.to_string()),
        Some(NEW_NAME.to_string()),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();

    let config = get_config(&mut app, addr);
    assert_eq!(
        Config {
            name: NEW_NAME.to_string(),
            description: NEW_DESCRIPTION.to_string(),
            owner: Addr::unchecked(ADDR1),
            lockdrop_contract: Addr::unchecked(NEW_LOCKDROP_ADDR),
            usdc_cl_pool_contract: Addr::unchecked(NEW_ORACLE_USDC_ADDR),
            atom_cl_pool_contract: Addr::unchecked(NEW_ORACLE_ATOM_ADDR),
        },
        config
    );
}

#[test]
#[should_panic(expected = "config description cannot be empty.")]
fn test_update_config_invalid_description() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    // Change name
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        Some(DAO_ADDR.to_string()),
        Some(LOCKDROP_ADDR.to_string()),
        Some(ORACLE_USDC_ADDR.to_string()),
        Some(ORACLE_ATOM_ADDR.to_string()),
        Some(NEW_NAME.to_string()),
        Some(String::from("")),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "config name cannot be empty.")]
fn test_update_config_invalid_name() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    // Change description
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        Some(DAO_ADDR.to_string()),
        Some(LOCKDROP_ADDR.to_string()),
        Some(ORACLE_USDC_ADDR.to_string()),
        Some(ORACLE_ATOM_ADDR.to_string()),
        Some(String::from("")),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();
}

#[test]
fn test_query_dao() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    let msg = QueryMsg::Dao {};
    let dao: Addr = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(dao, Addr::unchecked(DAO_ADDR));
}

#[test]
fn test_query_info() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    let msg = QueryMsg::Info {};
    let resp: InfoResponse = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(
        resp.info.contract,
        "crates.io:neutron-lockdrop-vault-for-cl-pools-for-cl-pools"
    );
}

#[test]
fn test_query_get_config() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            usdc_cl_pool_contract: ORACLE_USDC_ADDR.to_string(),
            atom_cl_pool_contract: ORACLE_ATOM_ADDR.to_string(),
        },
    );

    let config = get_config(&mut app, addr);
    assert_eq!(
        config,
        Config {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Addr::unchecked(DAO_ADDR),
            lockdrop_contract: Addr::unchecked(LOCKDROP_ADDR),
            usdc_cl_pool_contract: Addr::unchecked(ORACLE_USDC_ADDR),
            atom_cl_pool_contract: Addr::unchecked(ORACLE_ATOM_ADDR),
        }
    )
}

#[test]
pub fn test_migrate_update_version() {
    let mut deps = mock_dependencies();
    cw2::set_contract_version(&mut deps.storage, "my-contract", "old-version").unwrap();

    migrate(deps.as_mut(), mock_env(), MigrateMsg {}).unwrap();
    let version = cw2::get_contract_version(&deps.storage).unwrap();

    assert_eq!(version.version, CONTRACT_VERSION);
    assert_eq!(version.contract, CONTRACT_NAME);
}
