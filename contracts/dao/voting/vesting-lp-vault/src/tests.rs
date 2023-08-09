use crate::contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{coins, Addr, Coin, Empty, Uint128};
use cw_multi_test::{custom_app, App, AppResponse, Contract, ContractWrapper, Executor};
use cwd_interface::voting::{
    InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use neutron_vesting_lp_vault::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    types::Config,
};

const DAO_ADDR: &str = "dao";
const NAME: &str = "name";
const NEW_NAME: &str = "new_name";
const DESCRIPTION: &str = "description";
const NEW_DESCRIPTION: &str = "new description";
const ATOM_VESTING_LP_ADDR: &str = "atom_vesting_lp";
const USDC_VESTING_LP_ADDR: &str = "usdc_vesting_lp";
const ATOM_ORACLE_ADDR: &str = "atom_oracle";
const USDC_ORACLE_ADDR: &str = "usdc_oracle";
const NEW_ATOM_VESTING_LP_ADDR: &str = "new_atom_vesting_lp";
const NEW_USDC_VESTING_LP_ADDR: &str = "new_usdc_vesting_lp";
const NEW_ATOM_ORACLE_ADDR: &str = "new_atom_oracle";
const NEW_USDC_ORACLE_ADDR: &str = "new_usdc_oracle";
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

fn vesting_lp_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        vesting_lp::contract::execute,
        vesting_lp::contract::instantiate,
        vesting_lp::contract::query,
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

fn instantiate_vault(app: &mut App, id: u64, msg: InstantiateMsg) -> Addr {
    app.instantiate_contract(id, Addr::unchecked(DAO_ADDR), &msg, &[], "vault", None)
        .unwrap()
}

fn instantiate_vesting_lp(app: &mut App, id: u64, msg: vesting_lp::msg::InstantiateMsg) -> Addr {
    app.instantiate_contract(id, Addr::unchecked(DAO_ADDR), &msg, &[], "vesting_lp", None)
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
    owner: String,
    atom_vesting_lp_contract: String,
    atom_oracle_contract: String,
    usdc_vesting_lp_contract: String,
    usdc_oracle_contract: String,
    name: String,
    description: String,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::UpdateConfig {
            owner,
            atom_vesting_lp_contract,
            atom_oracle_contract,
            usdc_vesting_lp_contract,
            usdc_oracle_contract,
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
    // Populated fields
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
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
            owner: DAO_ADDR.to_string(),
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
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
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
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
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
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
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
        },
    );

    // From ADDR2, so not owner
    update_config(
        &mut app,
        addr,
        ADDR2,
        ADDR1.to_string(),
        NEW_ATOM_VESTING_LP_ADDR.to_string(),
        NEW_ATOM_ORACLE_ADDR.to_string(),
        NEW_USDC_VESTING_LP_ADDR.to_string(),
        NEW_USDC_ORACLE_ADDR.to_string(),
        NEW_NAME.to_string(),
        NEW_DESCRIPTION.to_string(),
    )
    .unwrap();
}

#[test]
fn test_update_config() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
        },
    );

    // Change owner, description, name, lp-vesting and oracle contracts
    update_config(
        &mut app,
        addr.clone(),
        DAO_ADDR,
        ADDR1.to_string(),
        NEW_ATOM_VESTING_LP_ADDR.to_string(),
        NEW_ATOM_ORACLE_ADDR.to_string(),
        NEW_USDC_VESTING_LP_ADDR.to_string(),
        NEW_USDC_ORACLE_ADDR.to_string(),
        NEW_NAME.to_string(),
        NEW_DESCRIPTION.to_string(),
    )
    .unwrap();

    let config = get_config(&mut app, addr);
    assert_eq!(
        Config {
            name: NEW_NAME.to_string(),
            description: NEW_DESCRIPTION.to_string(),
            owner: Addr::unchecked(ADDR1),
            atom_vesting_lp_contract: Addr::unchecked(NEW_ATOM_VESTING_LP_ADDR),
            atom_oracle_contract: Addr::unchecked(NEW_ATOM_ORACLE_ADDR),
            usdc_vesting_lp_contract: Addr::unchecked(NEW_USDC_VESTING_LP_ADDR),
            usdc_oracle_contract: Addr::unchecked(NEW_USDC_ORACLE_ADDR),
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
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
        },
    );

    // Change name
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        DAO_ADDR.to_string(),
        ATOM_VESTING_LP_ADDR.to_string(),
        ATOM_ORACLE_ADDR.to_string(),
        USDC_VESTING_LP_ADDR.to_string(),
        USDC_ORACLE_ADDR.to_string(),
        NEW_NAME.to_string(),
        String::from(""),
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
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
        },
    );

    // Change description
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        DAO_ADDR.to_string(),
        ATOM_VESTING_LP_ADDR.to_string(),
        ATOM_ORACLE_ADDR.to_string(),
        USDC_VESTING_LP_ADDR.to_string(),
        USDC_ORACLE_ADDR.to_string(),
        String::from(""),
        NEW_DESCRIPTION.to_string(),
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
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
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
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
        },
    );

    let msg = QueryMsg::Info {};
    let resp: InfoResponse = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(resp.info.contract, "crates.io:neutron-vesting-lp-vault");
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
            atom_vesting_lp_contract: ATOM_VESTING_LP_ADDR.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: USDC_VESTING_LP_ADDR.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
        },
    );

    let config = get_config(&mut app, addr);
    assert_eq!(
        config,
        Config {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Addr::unchecked(DAO_ADDR),
            atom_vesting_lp_contract: Addr::unchecked(ATOM_VESTING_LP_ADDR),
            atom_oracle_contract: Addr::unchecked(ATOM_ORACLE_ADDR),
            usdc_vesting_lp_contract: Addr::unchecked(USDC_VESTING_LP_ADDR),
            usdc_oracle_contract: Addr::unchecked(USDC_ORACLE_ADDR),
        }
    )
}

#[test]
fn test_voting_power_at_height() {
    let mut app = mock_app();

    let vesting_lp_id = app.store_code(vesting_lp_contract());
    let atom_vesting_lp_addr = instantiate_vesting_lp(
        &mut app,
        vesting_lp_id,
        vesting_lp::msg::InstantiateMsg {
            owner: DAO_ADDR.to_string(),
            vesting_managers: vec![],
            token_info_manager: "manager".to_string(),
        },
    );
    let usdc_vesting_lp_addr = instantiate_vesting_lp(
        &mut app,
        vesting_lp_id,
        vesting_lp::msg::InstantiateMsg {
            owner: DAO_ADDR.to_string(),
            vesting_managers: vec![],
            token_info_manager: "manager".to_string(),
        },
    );

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            atom_vesting_lp_contract: atom_vesting_lp_addr.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: usdc_vesting_lp_addr.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
        },
    );

    // describe test when lockdrop contract is implemented. use neutron vault tests as template.
    let resp = get_voting_power_at_height(&mut app, addr, ADDR1.to_string(), None);
    assert!(resp.power.is_zero());
}

#[test]
fn test_total_power_at_height() {
    let mut app = mock_app();

    let vesting_lp_id = app.store_code(vesting_lp_contract());
    let atom_vesting_lp_addr = instantiate_vesting_lp(
        &mut app,
        vesting_lp_id,
        vesting_lp::msg::InstantiateMsg {
            owner: DAO_ADDR.to_string(),
            vesting_managers: vec![],
            token_info_manager: "manager".to_string(),
        },
    );
    let usdc_vesting_lp_addr = instantiate_vesting_lp(
        &mut app,
        vesting_lp_id,
        vesting_lp::msg::InstantiateMsg {
            owner: DAO_ADDR.to_string(),
            vesting_managers: vec![],
            token_info_manager: "manager".to_string(),
        },
    );

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            atom_vesting_lp_contract: atom_vesting_lp_addr.to_string(),
            atom_oracle_contract: ATOM_ORACLE_ADDR.to_string(),
            usdc_vesting_lp_contract: usdc_vesting_lp_addr.to_string(),
            usdc_oracle_contract: USDC_ORACLE_ADDR.to_string(),
        },
    );

    // describe test when lockdrop contract is implemented. use neutron vault tests as template.
    let resp = get_total_power_at_height(&mut app, addr, None);
    assert!(resp.power.is_zero());
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
