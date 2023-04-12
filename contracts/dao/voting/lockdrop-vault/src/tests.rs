use crate::contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION};
use astroport::asset::AssetInfo;
use astroport::oracle::QueryMsg as OracleQueryMsg;
use astroport_periphery::lockdrop::{PoolType, QueryMsg as LockdropQueryMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{
    coins, to_binary, Addr, Binary, Coin, Decimal256, Deps, Empty, Env, Response, StdResult,
    Uint128,
};
use cw_multi_test::{custom_app, App, AppResponse, Contract, ContractWrapper, Executor};
use cwd_interface::voting::{
    InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use cwd_interface::Admin;
use neutron_lockdrop_vault::error::ContractError;
use neutron_lockdrop_vault::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use neutron_lockdrop_vault::types::Config;
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

fn lockdrop_query(_deps: Deps, _env: Env, msg: LockdropQueryMsg) -> StdResult<Binary> {
    match msg {
        LockdropQueryMsg::QueryUserLockupTotalAtHeight {
            pool_type,
            user_address: _,
            height: _,
        } => {
            let response = match pool_type {
                PoolType::ATOM => Uint128::from(1000u64),
                PoolType::USDC => Uint128::from(2000u64),
            };

            to_binary(&response)
        }
        LockdropQueryMsg::QueryLockupTotalAtHeight {
            pool_type,
            height: _,
        } => {
            let response = match pool_type {
                PoolType::ATOM => Uint128::from(3000u64),
                PoolType::USDC => Uint128::from(4000u64),
            };

            to_binary(&response)
        }
        _ => unreachable!(),
    }
}

fn lockdrop_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        LockdropQueryMsg,
        ContractError,
        ContractError,
        cosmwasm_std::StdError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        lockdrop_query,
    );
    Box::new(contract)
}

fn instantiate_lockdrop_contract(app: &mut App) -> Addr {
    let contract_id = app.store_code(lockdrop_contract());
    app.instantiate_contract(
        contract_id,
        Addr::unchecked(DAO_ADDR),
        &EmptyMsg {},
        &[],
        "lockdrop contract",
        None,
    )
    .unwrap()
}

fn oracle_query(_deps: Deps, _env: Env, msg: OracleQueryMsg) -> StdResult<Binary> {
    match msg {
        OracleQueryMsg::TWAPAtHeight { token, height: _ } => {
            let twap = match token.clone() {
                AssetInfo::NativeToken { denom } => match denom.as_str() {
                    "untrn" => Decimal256::from_atomics(4u64, 6).unwrap(),
                    _ => Decimal256::from_atomics(0u64, 6).unwrap(),
                },
                AssetInfo::Token { contract_addr: _ } => Decimal256::from_atomics(0u64, 6).unwrap(),
            };

            let response = vec![(token, twap)];
            to_binary(&response)
        }
        _ => unreachable!(),
    }
}

fn oracle_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        OracleQueryMsg,
        ContractError,
        ContractError,
        cosmwasm_std::StdError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        oracle_query,
    );
    Box::new(contract)
}

fn instantiate_oracle_contract(app: &mut App) -> Addr {
    let contract_id = app.store_code(oracle_contract());
    app.instantiate_contract(
        contract_id,
        Addr::unchecked(DAO_ADDR),
        &EmptyMsg {},
        &[],
        "oracle contract",
        None,
    )
    .unwrap()
}

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
    manager: Option<String>,
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
            manager,
            name,
            description,
        },
        &[],
    )
}

fn get_voting_power_at_height(
    app: &mut App,
    contract_addr: Addr,
    address: String,
    height: Option<u64>,
) -> VotingPowerAtHeightResponse {
    app.wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::VotingPowerAtHeight { address, height },
        )
        .unwrap()
}

fn get_total_power_at_height(
    app: &mut App,
    contract_addr: Addr,
    height: Option<u64>,
) -> TotalPowerAtHeightResponse {
    app.wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::TotalPowerAtHeight { height })
        .unwrap()
}

fn get_config(app: &mut App, contract_addr: Addr) -> Config {
    app.wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::GetConfig {})
        .unwrap()
}

fn get_description(app: &App, contract_addr: &Addr) -> String {
    app.wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Description {})
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
    // Populated fields
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::Address {
                addr: DAO_ADDR.to_string(),
            },
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );
    assert_eq!(get_dao(&app, &addr), String::from(DAO_ADDR));

    // Non populated fields
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::Address {
                addr: DAO_ADDR.to_string(),
            },
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: None,
        },
    );
    assert_eq!(get_dao(&app, &addr), String::from(DAO_ADDR));
}

#[test]
fn test_instantiate_dao_owner() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    // Populated fields
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    let config = get_config(&mut app, addr);

    assert_eq!(config.owner, Addr::unchecked(DAO_ADDR))
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
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
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
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    unbond_tokens(&mut app, addr, ADDR1, 100).unwrap();
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_update_config_invalid_sender() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    // From ADDR2, so not owner or manager
    update_config(
        &mut app,
        addr,
        ADDR2,
        Some(ADDR1.to_string()),
        Some(NEW_LOCKDROP_ADDR.to_string()),
        Some(NEW_ORACLE_USDC_ADDR.to_string()),
        Some(NEW_ORACLE_ATOM_ADDR.to_string()),
        Some(DAO_ADDR.to_string()),
        Some(NEW_NAME.to_string()),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "Only owner can change owner")]
fn test_update_config_non_owner_changes_owner() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    // ADDR1 is the manager so cannot change the owner
    update_config(
        &mut app,
        addr,
        ADDR1,
        Some(ADDR2.to_string()),
        Some(LOCKDROP_ADDR.to_string()),
        Some(ORACLE_USDC_ADDR.to_string()),
        Some(ORACLE_ATOM_ADDR.to_string()),
        None,
        Some(NAME.to_string()),
        Some(DESCRIPTION.to_string()),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "Only owner can change lockdrop contract")]
fn test_update_config_non_owner_changes_lockdrop() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    // ADDR1 is the manager so cannot change the lockdrop contract
    update_config(
        &mut app,
        addr,
        ADDR1,
        Some(DAO_ADDR.to_string()),
        Some(NEW_LOCKDROP_ADDR.to_string()),
        Some(ORACLE_USDC_ADDR.to_string()),
        Some(ORACLE_ATOM_ADDR.to_string()),
        None,
        Some(NAME.to_string()),
        Some(DESCRIPTION.to_string()),
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
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    // Swap owner and manager, change description, name and lockdrop contract
    update_config(
        &mut app,
        addr.clone(),
        DAO_ADDR,
        Some(ADDR1.to_string()),
        Some(NEW_LOCKDROP_ADDR.to_string()),
        Some(NEW_ORACLE_USDC_ADDR.to_string()),
        Some(NEW_ORACLE_ATOM_ADDR.to_string()),
        Some(DAO_ADDR.to_string()),
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
            oracle_usdc_contract: Addr::unchecked(NEW_ORACLE_USDC_ADDR),
            oracle_atom_contract: Addr::unchecked(NEW_ORACLE_ATOM_ADDR),
            manager: Some(Addr::unchecked(DAO_ADDR)),
        },
        config
    );
}

#[test]
fn test_update_config_as_manager() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    let description_before = get_description(&app, &addr);

    // Change description, name and manager as manager cannot change owner and lockdrop contract
    update_config(
        &mut app,
        addr.clone(),
        ADDR1,
        Some(DAO_ADDR.to_string()),
        Some(LOCKDROP_ADDR.to_string()),
        Some(ORACLE_USDC_ADDR.to_string()),
        Some(ORACLE_ATOM_ADDR.to_string()),
        Some(ADDR2.to_string()),
        Some(NEW_NAME.to_string()),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();

    let description_after = get_description(&app, &addr);
    assert_ne!(description_before, description_after);

    let config = get_config(&mut app, addr);
    assert_eq!(
        Config {
            name: NEW_NAME.to_string(),
            description: NEW_DESCRIPTION.to_string(),
            owner: Addr::unchecked(DAO_ADDR),
            lockdrop_contract: Addr::unchecked(LOCKDROP_ADDR),
            oracle_usdc_contract: Addr::unchecked(ORACLE_USDC_ADDR),
            oracle_atom_contract: Addr::unchecked(ORACLE_ATOM_ADDR),
            manager: Some(Addr::unchecked(ADDR2)),
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
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    // Change name and manager as manager cannot change owner
    update_config(
        &mut app,
        addr,
        ADDR1,
        Some(DAO_ADDR.to_string()),
        Some(LOCKDROP_ADDR.to_string()),
        Some(ORACLE_USDC_ADDR.to_string()),
        Some(ORACLE_ATOM_ADDR.to_string()),
        Some(ADDR2.to_string()),
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
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    // Change description and manager as manager cannot change owner
    update_config(
        &mut app,
        addr,
        ADDR1,
        Some(DAO_ADDR.to_string()),
        Some(LOCKDROP_ADDR.to_string()),
        Some(ORACLE_USDC_ADDR.to_string()),
        Some(ORACLE_ATOM_ADDR.to_string()),
        Some(ADDR2.to_string()),
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
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
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
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    let msg = QueryMsg::Info {};
    let resp: InfoResponse = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(resp.info.contract, "crates.io:neutron-lockdrop-vault");
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
            owner: Admin::CoreModule {},
            lockdrop_contract: LOCKDROP_ADDR.to_string(),
            oracle_usdc_contract: ORACLE_USDC_ADDR.to_string(),
            oracle_atom_contract: ORACLE_ATOM_ADDR.to_string(),
            manager: Some(ADDR1.to_string()),
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
            oracle_usdc_contract: Addr::unchecked(ORACLE_USDC_ADDR),
            oracle_atom_contract: Addr::unchecked(ORACLE_ATOM_ADDR),
            manager: Some(Addr::unchecked(ADDR1)),
        }
    )
}

#[test]
fn test_voting_power_at_height() {
    let mut app = mock_app();

    let lockdrop_contract = instantiate_lockdrop_contract(&mut app);
    let oracle_usdc_contract = instantiate_oracle_contract(&mut app);
    let oracle_atom_contract = instantiate_oracle_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::CoreModule {},
            lockdrop_contract: lockdrop_contract.to_string(),
            oracle_usdc_contract: oracle_usdc_contract.to_string(),
            oracle_atom_contract: oracle_atom_contract.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    let resp = get_voting_power_at_height(&mut app, addr, ADDR1.to_string(), None);
    assert_eq!(resp.power, Uint128::from(6u128));
}

#[test]
fn test_total_power_at_height() {
    let mut app = mock_app();

    let lockdrop_contract = instantiate_lockdrop_contract(&mut app);
    let oracle_usdc_contract = instantiate_oracle_contract(&mut app);
    let oracle_atom_contract = instantiate_oracle_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Admin::CoreModule {},
            lockdrop_contract: lockdrop_contract.to_string(),
            oracle_usdc_contract: oracle_usdc_contract.to_string(),
            oracle_atom_contract: oracle_atom_contract.to_string(),
            manager: Some(ADDR1.to_string()),
        },
    );

    let resp = get_total_power_at_height(&mut app, addr, None);
    assert_eq!(resp.power, Uint128::from(14u128));
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
