use crate::contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::Config;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{coins, Addr, Coin, Empty, Uint128};
use cw_multi_test::{
    custom_app, next_block, App, AppResponse, Contract, ContractWrapper, Executor,
};
use cwd_interface::voting::{
    BondingStatusResponse, InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use cwd_voting::vault::{BonderBalanceResponse, ListBondersResponse};

const DAO_ADDR: &str = "dao";
const NAME: &str = "name";
const NEW_NAME: &str = "new_name";
const DESCRIPTION: &str = "description";
const NEW_DESCRIPTION: &str = "new description";
const ADDR1: &str = "addr1";
const ADDR2: &str = "addr2";
const DENOM: &str = "ujuno";
const INVALID_DENOM: &str = "uinvalid";
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
                vec![
                    Coin {
                        denom: DENOM.to_string(),
                        amount: INIT_BALANCE,
                    },
                    Coin {
                        denom: INVALID_DENOM.to_string(),
                        amount: INIT_BALANCE,
                    },
                ],
            )
            .unwrap();
        r.bank
            .init_balance(
                s,
                &Addr::unchecked(ADDR1),
                vec![
                    Coin {
                        denom: DENOM.to_string(),
                        amount: INIT_BALANCE,
                    },
                    Coin {
                        denom: INVALID_DENOM.to_string(),
                        amount: INIT_BALANCE,
                    },
                ],
            )
            .unwrap();
        r.bank
            .init_balance(
                s,
                &Addr::unchecked(ADDR2),
                vec![
                    Coin {
                        denom: DENOM.to_string(),
                        amount: INIT_BALANCE,
                    },
                    Coin {
                        denom: INVALID_DENOM.to_string(),
                        amount: INIT_BALANCE,
                    },
                ],
            )
            .unwrap();
    })
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

fn update_config(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    owner: String,
    name: String,
    description: String,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::UpdateConfig {
            owner,
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

fn get_balance(app: &mut App, address: &str, denom: &str) -> Uint128 {
    app.wrap().query_balance(address, denom).unwrap().amount
}

fn get_bonding_status(app: &App, contract_addr: &Addr, address: &str) -> BondingStatusResponse {
    app.wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::BondingStatus {
                address: address.to_string(),
                height: None,
            },
        )
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
            denom: DENOM.to_string(),
        },
    );
    assert_eq!(get_dao(&app, &addr), String::from(DAO_ADDR));
}

#[test]
#[should_panic(expected = "Must send reserve token 'ujuno'")]
fn test_bond_invalid_denom() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            denom: DENOM.to_string(),
        },
    );

    // Try and bond an invalid denom
    bond_tokens(&mut app, addr, ADDR1, 100, INVALID_DENOM).unwrap();
}

#[test]
fn test_bond_valid_denom() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            denom: DENOM.to_string(),
        },
    );

    let mut bonding_status: BondingStatusResponse = get_bonding_status(&app, &addr, ADDR1);
    assert!(bonding_status.bonding_enabled);
    assert_eq!(bonding_status.unbondable_abount, Uint128::zero());

    // Try and bond an valid denom
    bond_tokens(&mut app, addr.clone(), ADDR1, 100, DENOM).unwrap();
    app.update_block(next_block);

    bonding_status = get_bonding_status(&app, &addr, ADDR1);
    assert!(bonding_status.bonding_enabled);
    assert_eq!(bonding_status.unbondable_abount, Uint128::from(100u32));
}

#[test]
#[should_panic(expected = "Can only unbond less than or equal to the amount you have bonded")]
fn test_unbond_none_bonded() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            denom: DENOM.to_string(),
        },
    );

    unbond_tokens(&mut app, addr, ADDR1, 100).unwrap();
}

#[test]
#[should_panic(expected = "Can only unbond less than or equal to the amount you have bonded")]
fn test_unbond_invalid_balance() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            denom: DENOM.to_string(),
        },
    );

    // bond some tokens
    bond_tokens(&mut app, addr.clone(), ADDR1, 100, DENOM).unwrap();
    app.update_block(next_block);

    // Try and unbond too many
    unbond_tokens(&mut app, addr, ADDR1, 200).unwrap();
}

#[test]
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
            denom: DENOM.to_string(),
        },
    );

    assert_eq!(get_balance(&mut app, ADDR1, DENOM), INIT_BALANCE);
    // bond some tokens
    bond_tokens(&mut app, addr.clone(), ADDR1, 100, DENOM).unwrap();
    app.update_block(next_block);
    assert_eq!(get_balance(&mut app, ADDR1, DENOM), Uint128::new(9900));

    let mut bonding_status: BondingStatusResponse = get_bonding_status(&app, &addr, ADDR1);
    assert!(bonding_status.bonding_enabled);
    assert_eq!(bonding_status.unbondable_abount, Uint128::from(100u32));

    // Unbond some
    unbond_tokens(&mut app, addr.clone(), ADDR1, 75).unwrap();
    assert_eq!(get_balance(&mut app, ADDR1, DENOM), Uint128::new(9975));
    app.update_block(next_block);

    bonding_status = get_bonding_status(&app, &addr, ADDR1);
    assert!(bonding_status.bonding_enabled);
    assert_eq!(bonding_status.unbondable_abount, Uint128::from(25u32));

    // Unbond the rest
    unbond_tokens(&mut app, addr.clone(), ADDR1, 25).unwrap();
    assert_eq!(get_balance(&mut app, ADDR1, DENOM), INIT_BALANCE);
    app.update_block(next_block);

    bonding_status = get_bonding_status(&app, &addr, ADDR1);
    assert!(bonding_status.bonding_enabled);
    assert_eq!(bonding_status.unbondable_abount, Uint128::zero());
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
            denom: DENOM.to_string(),
        },
    );

    // From ADDR2, so not owner
    update_config(
        &mut app,
        addr,
        ADDR2,
        ADDR1.to_string(),
        NEW_NAME.to_string(),
        NEW_DESCRIPTION.to_string(),
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
            denom: DENOM.to_string(),
        },
    );

    // Change owner, description and name
    update_config(
        &mut app,
        addr.clone(),
        DAO_ADDR,
        ADDR1.to_string(),
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
            denom: DENOM.to_string(),
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
            denom: DENOM.to_string(),
        },
    );

    // Change name
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        DAO_ADDR.to_string(),
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
            denom: DENOM.to_string(),
        },
    );

    // Change description
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        DAO_ADDR.to_string(),
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
            denom: DENOM.to_string(),
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
            denom: DENOM.to_string(),
        },
    );

    let msg = QueryMsg::Info {};
    let resp: InfoResponse = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(resp.info.contract, "crates.io:neutron-voting-vault");
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
            denom: DENOM.to_string(),
        },
    );

    let config = get_config(&mut app, addr);
    assert_eq!(
        config,
        Config {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: Addr::unchecked(DAO_ADDR),
            denom: DENOM.to_string(),
        }
    )
}

#[test]
fn test_voting_power_queries() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            denom: DENOM.to_string(),
        },
    );

    // Total power is 0
    let resp = get_total_power_at_height(&mut app, addr.clone(), None);
    assert!(resp.power.is_zero());

    // ADDR1 has no power, none bonded
    let resp = get_voting_power_at_height(&mut app, addr.clone(), ADDR1.to_string(), None);
    assert!(resp.power.is_zero());

    // ADDR1 bonds
    bond_tokens(&mut app, addr.clone(), ADDR1, 100, DENOM).unwrap();
    app.update_block(next_block);

    // Total power is 100
    let resp = get_total_power_at_height(&mut app, addr.clone(), None);
    assert_eq!(resp.power, Uint128::new(100));

    // ADDR1 has 100 power
    let resp = get_voting_power_at_height(&mut app, addr.clone(), ADDR1.to_string(), None);
    assert_eq!(resp.power, Uint128::new(100));

    // ADDR2 still has 0 power
    let resp = get_voting_power_at_height(&mut app, addr.clone(), ADDR2.to_string(), None);
    assert!(resp.power.is_zero());

    // ADDR2 bonds
    bond_tokens(&mut app, addr.clone(), ADDR2, 50, DENOM).unwrap();
    app.update_block(next_block);
    let prev_height = app.block_info().height - 1;

    // Query the previous height, total 100, ADDR1 100, ADDR2 0
    // Total power is 100
    let resp = get_total_power_at_height(&mut app, addr.clone(), Some(prev_height));
    assert_eq!(resp.power, Uint128::new(100));

    // ADDR1 has 100 power
    let resp =
        get_voting_power_at_height(&mut app, addr.clone(), ADDR1.to_string(), Some(prev_height));
    assert_eq!(resp.power, Uint128::new(100));

    // ADDR2 still has 0 power
    let resp =
        get_voting_power_at_height(&mut app, addr.clone(), ADDR2.to_string(), Some(prev_height));
    assert!(resp.power.is_zero());

    // For current height, total 150, ADDR1 100, ADDR2 50
    // Total power is 150
    let resp = get_total_power_at_height(&mut app, addr.clone(), None);
    assert_eq!(resp.power, Uint128::new(150));

    // ADDR1 has 100 power
    let resp = get_voting_power_at_height(&mut app, addr.clone(), ADDR1.to_string(), None);
    assert_eq!(resp.power, Uint128::new(100));

    // ADDR2 now has 50 power
    let resp = get_voting_power_at_height(&mut app, addr.clone(), ADDR2.to_string(), None);
    assert_eq!(resp.power, Uint128::new(50));

    // ADDR1 unbonds half
    unbond_tokens(&mut app, addr.clone(), ADDR1, 50).unwrap();
    app.update_block(next_block);
    let prev_height = app.block_info().height - 1;

    // Query the previous height, total 150, ADDR1 100, ADDR2 50
    // Total power is 100
    let resp = get_total_power_at_height(&mut app, addr.clone(), Some(prev_height));
    assert_eq!(resp.power, Uint128::new(150));

    // ADDR1 has 100 power
    let resp =
        get_voting_power_at_height(&mut app, addr.clone(), ADDR1.to_string(), Some(prev_height));
    assert_eq!(resp.power, Uint128::new(100));

    // ADDR2 still has 0 power
    let resp =
        get_voting_power_at_height(&mut app, addr.clone(), ADDR2.to_string(), Some(prev_height));
    assert_eq!(resp.power, Uint128::new(50));

    // For current height, total 100, ADDR1 50, ADDR2 50
    // Total power is 100
    let resp = get_total_power_at_height(&mut app, addr.clone(), None);
    assert_eq!(resp.power, Uint128::new(100));

    // ADDR1 has 50 power
    let resp = get_voting_power_at_height(&mut app, addr.clone(), ADDR1.to_string(), None);
    assert_eq!(resp.power, Uint128::new(50));

    // ADDR2 now has 50 power
    let resp = get_voting_power_at_height(&mut app, addr, ADDR2.to_string(), None);
    assert_eq!(resp.power, Uint128::new(50));
}

#[test]
fn test_query_list_bonders() {
    let mut app = mock_app();
    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            denom: DENOM.to_string(),
        },
    );

    // ADDR1 bonds
    bond_tokens(&mut app, addr.clone(), ADDR1, 100, DENOM).unwrap();

    // ADDR2 bonds
    bond_tokens(&mut app, addr.clone(), ADDR2, 50, DENOM).unwrap();

    // check entire result set
    let bonders: ListBondersResponse = app
        .wrap()
        .query_wasm_smart(
            addr.clone(),
            &QueryMsg::ListBonders {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    let test_res = ListBondersResponse {
        bonders: vec![
            BonderBalanceResponse {
                address: ADDR1.to_string(),
                balance: Uint128::new(100),
            },
            BonderBalanceResponse {
                address: ADDR2.to_string(),
                balance: Uint128::new(50),
            },
        ],
    };

    assert_eq!(bonders, test_res);

    // skipped 1, check result
    let bonders: ListBondersResponse = app
        .wrap()
        .query_wasm_smart(
            addr.clone(),
            &QueryMsg::ListBonders {
                start_after: Some(ADDR1.to_string()),
                limit: None,
            },
        )
        .unwrap();

    let test_res = ListBondersResponse {
        bonders: vec![BonderBalanceResponse {
            address: ADDR2.to_string(),
            balance: Uint128::new(50),
        }],
    };

    assert_eq!(bonders, test_res);

    // skipped 2, check result. should be nothing
    let bonders: ListBondersResponse = app
        .wrap()
        .query_wasm_smart(
            addr,
            &QueryMsg::ListBonders {
                start_after: Some(ADDR2.to_string()),
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(bonders, ListBondersResponse { bonders: vec![] });
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
