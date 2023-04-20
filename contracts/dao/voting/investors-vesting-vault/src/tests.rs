use crate::contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::Config;
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Empty, Env, Response, StdResult, Uint128};
use cw_multi_test::{custom_app, App, AppResponse, Contract, ContractWrapper, Executor};
use cwd_interface::voting::{
    InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

fn vesting_query(_deps: Deps, _env: Env, msg: vesting_base::msg::QueryMsg) -> StdResult<Binary> {
    match msg {
        vesting_base::msg::QueryMsg::HistoricalExtension {
            msg:
                vesting_base::msg::QueryMsgHistorical::UnclaimedAmountAtHeight {
                    address: _,
                    height: _,
                },
        } => {
            let response = Uint128::from(10000u64);
            to_binary(&response)
        }
        vesting_base::msg::QueryMsg::HistoricalExtension {
            msg: vesting_base::msg::QueryMsgHistorical::UnclaimedTotalAmountAtHeight { height: _ },
        } => {
            let response = Uint128::from(10000u64);
            to_binary(&response)
        }
        _ => unimplemented!(),
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct EmptyMsg {}

fn vesting_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        vesting_base::msg::QueryMsg,
        ContractError,
        ContractError,
        cosmwasm_std::StdError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        vesting_query,
    );
    Box::new(contract)
}

fn instantiate_vesting_contract(app: &mut App) -> Addr {
    let contract_id = app.store_code(vesting_contract());
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
    vesting_contract_address: Option<String>,
    owner: String,
    description: Option<String>,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::UpdateConfig {
            vesting_contract_address,
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
            vesting_contract_address: vesting_contract.to_string(),
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
            vesting_contract_address: vesting_contract.to_string(),
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
            vesting_contract_address: vesting_contract.to_string(),
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
            vesting_contract_address: Addr::unchecked(vesting_contract),
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
            vesting_contract_address: vesting_contract.to_string(),
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
            vesting_contract_address: vesting_contract.to_string(),
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
            vesting_contract_address: vesting_contract.to_string(),
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
            vesting_contract_address: vesting_contract.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
            name: "vesting vault".to_string(),
        },
    );

    let config = get_config(&mut app, addr);
    assert_eq!(
        config,
        Config {
            vesting_contract_address: Addr::unchecked(vesting_contract),
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
            vesting_contract_address: vesting_contract.to_string(),
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
