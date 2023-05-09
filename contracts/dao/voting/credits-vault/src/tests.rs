use crate::contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION};
use crate::msg::{CreditsQueryMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, TotalSupplyResponse};
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
const AIRDROP_ADDR: &str = "airdrop";
const NAME: &str = "name";
const DESCRIPTION: &str = "description";
const NEW_NAME: &str = "new name";
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

fn credits_query(_deps: Deps, _env: Env, msg: CreditsQueryMsg) -> StdResult<Binary> {
    match msg {
        CreditsQueryMsg::BalanceAtHeight { address, height: _ } => {
            let response = cw20::BalanceResponse {
                balance: Uint128::from(if address == AIRDROP_ADDR {
                    2000u64
                } else {
                    6000u64
                }),
            };
            to_binary(&response)
        }
        CreditsQueryMsg::TotalSupplyAtHeight { height: _ } => {
            let response = TotalSupplyResponse {
                total_supply: Uint128::from(10000u64),
            };
            to_binary(&response)
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct EmptyMsg {}

fn credits_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        CreditsQueryMsg,
        ContractError,
        ContractError,
        cosmwasm_std::StdError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        credits_query,
    );
    Box::new(contract)
}

fn instantiate_credits_contract(app: &mut App) -> Addr {
    let contract_id = app.store_code(credits_contract());
    app.instantiate_contract(
        contract_id,
        Addr::unchecked(DAO_ADDR),
        &EmptyMsg {},
        &[],
        "credits contract",
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
    credits_contract_address: Option<String>,
    owner: Option<String>,
    name: Option<String>,
    description: Option<String>,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr,
        &ExecuteMsg::UpdateConfig {
            credits_contract_address,
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

fn get_description(app: &mut App, contract_addr: Addr) -> String {
    app.wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Description {})
        .unwrap()
}

#[test]
fn test_instantiate() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let _addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_update_config_unauthorized() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    // From ADDR2, so not owner
    update_config(
        &mut app,
        addr,
        ADDR2,
        Some(credits_contract.to_string()),
        Some(ADDR1.to_string()),
        Some(NEW_NAME.to_string()),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();
}

#[test]
fn test_update_config_as_owner() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    // Change owner and description
    update_config(
        &mut app,
        addr.clone(),
        DAO_ADDR,
        Some(credits_contract.to_string()),
        Some(ADDR1.to_string()),
        Some(NEW_NAME.to_string()),
        Some(NEW_DESCRIPTION.to_string()),
    )
    .unwrap();

    let config = get_config(&mut app, addr);
    assert_eq!(
        Config {
            credits_contract_address: Addr::unchecked(credits_contract),
            airdrop_contract_address: Addr::unchecked(AIRDROP_ADDR),
            name: NEW_NAME.to_string(),
            description: NEW_DESCRIPTION.to_string(),
            owner: Addr::unchecked(ADDR1),
        },
        config
    );
}

#[test]
#[should_panic(expected = "config description cannot be empty.")]
fn test_update_config_invalid_description() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    // Change description
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        None,
        None,
        None,
        Some(String::from("")),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "config name cannot be empty.")]
fn test_update_config_invalid_name() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    // Change description
    update_config(
        &mut app,
        addr,
        DAO_ADDR,
        None,
        None,
        Some(String::from("")),
        None,
    )
    .unwrap();
}

#[test]
fn test_query_dao() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    let msg = QueryMsg::Dao {};
    let dao: Addr = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(dao, Addr::unchecked(DAO_ADDR));
}

#[test]
fn test_query_info() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    let msg = QueryMsg::Info {};
    let resp: InfoResponse = app.wrap().query_wasm_smart(addr, &msg).unwrap();
    assert_eq!(resp.info.contract, "crates.io:neutron-credits-vault");
}

#[test]
fn test_query_get_config() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    let config = get_config(&mut app, addr);
    assert_eq!(
        config,
        Config {
            credits_contract_address: Addr::unchecked(credits_contract),
            airdrop_contract_address: Addr::unchecked(AIRDROP_ADDR),
            description: DESCRIPTION.to_string(),
            name: NAME.to_string(),
            owner: Addr::unchecked(DAO_ADDR),
        }
    )
}

#[test]
fn test_query_get_description() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    let description = get_description(&mut app, addr);
    assert_eq!(DESCRIPTION, description)
}

#[test]
fn test_voting_power_queries() {
    let mut app = mock_app();
    let credits_contract = instantiate_credits_contract(&mut app);

    let vault_id = app.store_code(vault_contract());
    let addr = instantiate_vault(
        &mut app,
        vault_id,
        InstantiateMsg {
            credits_contract_address: credits_contract.to_string(),
            airdrop_contract_address: AIRDROP_ADDR.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            owner: DAO_ADDR.to_string(),
        },
    );

    // Total power is 8000, total supply is 10000, airdrop has 2000, 10000 - 2000 = 8000
    let resp = get_total_power_at_height(&mut app, addr.clone(), None);
    assert_eq!(Uint128::from(8000u64), resp.power);

    // ADDR1 has 6000 voting power
    let resp = get_voting_power_at_height(&mut app, addr.clone(), ADDR1.to_string(), None);
    assert_eq!(Uint128::from(6000u64), resp.power);

    // AIRDROP_ADDR has 2000 credits tokens, but it still has 0 voting power
    let resp = get_voting_power_at_height(&mut app, addr, AIRDROP_ADDR.to_string(), None);
    assert_eq!(Uint128::from(0u64), resp.power);
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
