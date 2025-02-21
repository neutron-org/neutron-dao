use crate::contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, BLACKLISTED_ADDRESSES, CONFIG};
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Empty, Env, Response, StdResult, Uint128};
use cw_multi_test::{custom_app, App, AppResponse, Contract, ContractWrapper, Executor};
use cwd_interface::voting::{
    InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use neutron_staking_tracker::contract::{execute, query};

const DAO_ADDR: &str = "dao";
const DESCRIPTION: &str = "description";
const NEW_DESCRIPTION: &str = "new description";
const ADDR1: &str = "addr1";
const ADDR2: &str = "addr2";

fn vault_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn staking_tracker_query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VotingPowerAtHeight { .. } => {
            let response = Uint128::from(10000u64);
            to_json_binary(&response)
        }
        QueryMsg::TotalPowerAtHeight { .. } => {
            let response = Uint128::from(10000u64);
            to_json_binary(&response)
        }
        _ => unimplemented!(),
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct EmptyMsg {}

fn staking_tracker_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        QueryMsg,
        ContractError,
        ContractError,
        cosmwasm_std::StdError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        staking_tracker_query,
    );
    Box::new(contract)
}

fn instantiate_vesting_contract(app: &mut App) -> Addr {
    let contract_id = app.store_code(staking_tracker_contract());
    app.instantiate_contract(
        contract_id,
        Addr::unchecked(DAO_ADDR),
        &EmptyMsg {},
        &[],
        "vesting contract",
        None,
    )
    .unwrap()
}

fn mock_app() -> App {
    custom_app(|_r, _a, _s| {})
}

fn instantiate_vault(app: &mut App, id: u64, msg: InstantiateMsg) -> Addr {
    app.instantiate_contract(id, Addr::unchecked(DAO_ADDR), &msg, &[], "vault", None)
        .unwrap()
}

fn update_config(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    staking_tracker_contract_address: Option<String>,
    owner: String,
    description: Option<String>,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::UpdateConfig {
            staking_tracker_contract_address,
            owner: Some(owner),
            description,
            name: None,
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

#[test]
fn test_instantiate() {
    let mut app = mock_app();
    let vesting_contract = instantiate_vesting_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let _addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            staking_tracker_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_update_config_unauthorized() {
    let mut app = mock_app();
    let vesting_contract = instantiate_vesting_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            staking_tracker_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );

    // From ADDR2, so not owner
    update_config(
        &mut app,
        addr,
        ADDR2,
        Some(vesting_contract.to_string()),
        ADDR1.to_string(),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();
}

#[test]
fn test_update_config_as_owner() {
    let mut app = mock_app();
    let vesting_contract = instantiate_vesting_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            staking_tracker_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );

    // Change owner and description
    update_config(
        &mut app,
        addr.clone(),
        DAO_ADDR,
        Some(vesting_contract.to_string()),
        ADDR1.to_string(),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();

    let config = get_config(&mut app, addr);
    assert_eq!(
        Config {
            staking_tracker_contract_address: Addr::unchecked(vesting_contract),
            description: NEW_DESCRIPTION.to_string(),
            owner: Addr::unchecked(ADDR1),
            name: "vesting vault".to_string(),
        },
        config
    );
}

#[test]
#[should_panic(expected = "config description cannot be empty.")]
fn test_update_config_invalid_description() {
    let mut app = mock_app();
    let vesting_contract = instantiate_vesting_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            staking_tracker_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );

    // Change description
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        Some(vesting_contract.to_string()),
        DAO_ADDR.to_string(),
        Some(String::from("")),
    )
    .unwrap();
}

#[test]
fn test_query_dao() {
    let mut app = mock_app();
    let vesting_contract = instantiate_vesting_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            staking_tracker_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );

    let msg = QueryMsg::Dao {};
    let dao: Addr = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(dao, Addr::unchecked(DAO_ADDR));
}

#[test]
fn test_query_info() {
    let mut app = mock_app();
    let vesting_contract = instantiate_vesting_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            staking_tracker_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );

    let msg = QueryMsg::Info {};
    let resp: InfoResponse = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(
        resp.info.contract,
        "crates.io:neutron-investors-vesting-vault"
    );
}

#[test]
fn test_query_get_config() {
    let mut app = mock_app();
    let vesting_contract = instantiate_vesting_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            staking_tracker_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );

    let config = get_config(&mut app, addr);
    assert_eq!(
        config,
        Config {
            staking_tracker_contract_address: Addr::unchecked(vesting_contract),
            description: DESCRIPTION.to_string(),
            owner: Addr::unchecked(DAO_ADDR),
            name: "vesting vault".to_string(),
        }
    )
}

#[test]
fn test_voting_power_queries() {
    let mut app = mock_app();
    let vesting_contract = instantiate_vesting_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            staking_tracker_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );

    let resp = get_total_power_at_height(&mut app, addr.clone(), None);
    assert_eq!(Uint128::from(10000u64), resp.power);

    let resp = get_voting_power_at_height(&mut app, addr, ADDR1.to_string(), None);
    assert_eq!(Uint128::from(10000u64), resp.power);
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

#[test]
fn test_add_and_remove_from_blacklist() {
    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let admin = deps.api.addr_make("admin");
    let addr1 = deps.api.addr_make("addr1");
    let addr2 = deps.api.addr_make("addr2");

    // Initialize config with owner
    let config = crate::state::Config {
        name: String::from("Test Config"),
        description: String::from("Testing blacklist functionality"),
        staking_tracker_contract_address: deps.api.addr_make("staking_tracker_contract"),
        owner: admin.clone(),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Add addresses to the blacklist
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&admin, &[]),
        neutron_staking_tracker::msg::ExecuteMsg::AddToBlacklist {
            addresses: vec![String::from(addr1.clone()), String::from(addr2.clone())],
        },
    );
    assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

    // Verify that addresses are blacklisted
    let is_addr1_blacklisted = BLACKLISTED_ADDRESSES
        .load(deps.as_ref().storage, addr1.clone())
        .unwrap_or(false);
    let is_addr2_blacklisted = BLACKLISTED_ADDRESSES
        .load(deps.as_ref().storage, addr2.clone())
        .unwrap_or(false);
    assert!(is_addr1_blacklisted, "Address addr1 is not blacklisted");
    assert!(is_addr2_blacklisted, "Address addr2 is not blacklisted");

    // Remove addresses from the blacklist
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&admin, &[]),
        neutron_staking_tracker::msg::ExecuteMsg::RemoveFromBlacklist {
            addresses: vec![String::from(addr1.clone()), String::from(addr2.clone())],
        },
    );
    assert!(
        res.is_ok(),
        "Error removing from blacklist: {:?}",
        res.err()
    );

    // Verify that addresses are no longer blacklisted
    let is_addr1_blacklisted = BLACKLISTED_ADDRESSES
        .may_load(deps.as_ref().storage, addr1)
        .unwrap();
    let is_addr2_blacklisted = BLACKLISTED_ADDRESSES
        .may_load(deps.as_ref().storage, addr2)
        .unwrap();
    assert!(
        is_addr1_blacklisted.is_none(),
        "Address addr1 is still blacklisted"
    );
    assert!(
        is_addr2_blacklisted.is_none(),
        "Address addr2 is still blacklisted"
    );
}


#[test]
fn test_check_if_address_is_blacklisted() {
    let mut deps = dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let admin = deps.api.addr_make("admin");
    let addr1 = deps.api.addr_make("addr1");
    let addr2 = deps.api.addr_make("addr2");

    // Initialize config with owner
    let config = neutron_staking_tracker::state::Config {
        name: String::from("Test Config"),
        description: String::from("Testing blacklist functionality"),
        owner: admin.clone(),
        staking_proxy_info_contract_address: None,
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Add an address to the blacklist
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&admin, &[]),
        neutron_staking_tracker::msg::ExecuteMsg::AddToBlacklist {
            addresses: vec![addr1.to_string()],
        },
    );
    assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

    // Query if the address is blacklisted
    let query_res = query(
        deps.as_ref(),
        mock_env(),
        neutron_staking_tracker::msg::QueryMsg::IsAddressBlacklisted {
            address: addr1.to_string(),
        },
    );
    assert!(
        query_res.is_ok(),
        "Error querying blacklist status: {:?}",
        query_res.err()
    );

    let is_blacklisted: bool = from_json(query_res.unwrap()).unwrap();
    assert!(is_blacklisted, "Address addr1 should be blacklisted");

    // Query an address that is not blacklisted
    let query_res = query(
        deps.as_ref(),
        mock_env(),
        neutron_staking_tracker::msg::QueryMsg::IsAddressBlacklisted {
            address: addr2.to_string(),
        },
    );
    assert!(
        query_res.is_ok(),
        "Error querying blacklist status: {:?}",
        query_res.err()
    );

    let is_blacklisted: bool = from_json(query_res.unwrap()).unwrap();
    assert!(!is_blacklisted, "Address addr2 should not be blacklisted");
}


#[test]
fn test_total_vp_excludes_blacklisted_addresses() {
    let mut deps = dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let admin = deps.api.addr_make("admin");
    let delegator1 = deps.api.addr_make("delegator1");
    let delegator2 = deps.api.addr_make("delegator2");

    let env = mock_env();

    let config = neutron_staking_tracker::state::Config {
        name: "Test Vault".to_string(),
        description: "Testing vault functionality".to_string(),
        owner: admin.clone(),
        staking_proxy_info_contract_address: None,
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Define consensus and operator addresses
    let cons_addr1 = Addr::unchecked("neutronvalcons1xyz");
    let oper_addr1 = Addr::unchecked("neutronvaloper1xyz");
    let cons_addr2 = Addr::unchecked("neutronvalcons2xyz");
    let oper_addr2 = Addr::unchecked("neutronvaloper2xyz");

    // Add validators using consensus address as the key
    let validator1 = Validator {
        cons_address: cons_addr1.clone(),
        oper_address: oper_addr1.clone(),
        bonded: true,
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
        active: true,
    };
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &cons_addr1,
            &validator1,
            env.block.height,
        )
        .unwrap();

    let validator2 = Validator {
        cons_address: cons_addr2.clone(),
        oper_address: oper_addr2.clone(),
        bonded: true,
        total_tokens: Uint128::new(500),
        total_shares: Uint128::new(500),
        active: true,
    };
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &cons_addr2,
            &validator2,
            env.block.height,
        )
        .unwrap();

    let delegation1 = Delegation {
        delegator_address: delegator1.clone(),
        validator_address: oper_addr1.clone(), // Uses operator address
        shares: Uint128::new(500),
    };
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator1, &oper_addr1),
            &delegation1,
            env.block.height,
        )
        .unwrap();

    let delegation2 = Delegation {
        delegator_address: delegator2.clone(),
        validator_address: oper_addr2.clone(), // Uses operator address
        shares: Uint128::new(500),
    };
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator2, &oper_addr2),
            &delegation2,
            env.block.height,
        )
        .unwrap();

    // Query total voting power **before** blacklisting anything
    let initial_query_res = query(
        deps.as_ref(),
        env.clone(),
        neutron_staking_tracker::msg::QueryMsg::TotalStakeAtHeight {
            height: Some(env.block.height + 1),
        },
    );
    assert!(
        initial_query_res.is_ok(),
        "Error querying total power before blacklisting: {:?}",
        initial_query_res.err()
    );

    let initial_total_power: Uint128 = from_json(initial_query_res.unwrap()).unwrap();

    // Expected power: sum of both validator tokens (1000 + 500 = 1500)
    assert_eq!(
        initial_total_power,
        Uint128::new(1500),
        "Initial total power should be sum of both validators' tokens"
    );

    // Blacklist address "addr2"
    let res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&admin, &[]),
        neutron_staking_tracker::msg::ExecuteMsg::AddToBlacklist {
            addresses: vec![delegator2.to_string()],
        },
    );
    assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

    // Ensure validator1 still exists and has tokens
    let validator1_state = VALIDATORS.load(deps.as_ref().storage, &cons_addr1).unwrap();
    assert_eq!(
        validator1_state.total_tokens,
        Uint128::new(1000),
        "Validator1's tokens are incorrect"
    );

    // Ensure validator2 still exists
    let validator2_state = VALIDATORS.load(deps.as_ref().storage, &cons_addr2).unwrap();
    assert_eq!(
        validator2_state.total_tokens,
        Uint128::new(500),
        "Validator2's tokens are incorrect"
    );

    // Ensure delegation1 is still present
    let delegation1_state = DELEGATIONS
        .load(deps.as_ref().storage, (&delegator1, &oper_addr1))
        .unwrap();
    assert_eq!(
        delegation1_state.shares,
        Uint128::new(500),
        "Delegation1 shares incorrect"
    );

    // Ensure delegation2 is blacklisted correctly
    let is_blacklisted = BLACKLISTED_ADDRESSES
        .load(deps.as_ref().storage, delegator2.clone())
        .unwrap_or(false);
    assert!(is_blacklisted, "Delegator2 should be blacklisted");

    // Query total voting power **after** blacklisting
    let query_res = query(
        deps.as_ref(),
        env.clone(),
        neutron_staking_tracker::msg::QueryMsg::TotalStakeAtHeight {
            height: Some(env.block.height + 1),
        },
    );
    assert!(
        query_res.is_ok(),
        "Error querying total power after blacklisting: {:?}",
        query_res.err()
    );

    let total_power: Uint128 = from_json(query_res.unwrap()).unwrap();

    // Only validator1's power should count (1000), validator2's delegation is blacklisted
    assert_eq!(
        total_power,
        Uint128::new(1000),
        "Total power should exclude blacklisted address"
    );
}
