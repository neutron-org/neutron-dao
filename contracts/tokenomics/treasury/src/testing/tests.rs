use std::str::FromStr;

use cosmwasm_std::{
    coin, coins,
    testing::{mock_env, mock_info},
    to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Empty, Uint128, WasmMsg,
};

use crate::error::ContractError;
use crate::{
    contract::{execute, instantiate},
    msg::{DistributeMsg, ExecuteMsg, InstantiateMsg},
    state::{CONFIG, TOTAL_DISTRIBUTED, TOTAL_RECEIVED, TOTAL_RESERVED},
    testing::mock_querier::mock_dependencies,
};

const DENOM: &str = "denom";

pub fn init_base_contract(deps: DepsMut<Empty>, distribution_rate: &str) {
    let msg = InstantiateMsg {
        denom: DENOM.to_string(),
        min_period: 1000,
        distribution_contract: "distribution_contract".to_string(),
        reserve_contract: "reserve_contract".to_string(),
        distribution_rate: Decimal::from_str(distribution_rate).unwrap(),
        owner: "owner".to_string(),
        security_dao_address: "security_address_dao".to_string(),
    };
    let info = mock_info("creator", &coins(2, DENOM));
    instantiate(deps, mock_env(), info, msg).unwrap();
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut(), "0.23");
    let msg = ExecuteMsg::TransferOwnership("new_owner".to_string());
    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg);
    assert!(res.is_ok());
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config.main_dao_contract.to_string(),
        "new_owner".to_string()
    );
}

#[test]
fn test_collect_with_no_money() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut(), "1");
    let msg = ExecuteMsg::Distribute {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("anyone", &[]), msg);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::NoFundsToDistribute {});
}

#[test]
fn test_distribute_success() {
    let mut deps = mock_dependencies(&[coin(1000000, DENOM)]);
    init_base_contract(deps.as_mut(), "0.23");
    let msg = ExecuteMsg::Distribute {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("anyone", &[]), msg);
    assert!(res.is_ok());
    let messages = res.unwrap().messages;
    assert_eq!(messages.len(), 2);
    assert_eq!(
        messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "distribution_contract".to_string(),
            funds: vec![Coin {
                denom: DENOM.to_string(),
                amount: Uint128::from(230000u128)
            }],
            msg: to_binary(&DistributeMsg::Fund {}).unwrap(),
        })
    );
    assert_eq!(
        messages[1].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "reserve_contract".to_string(),
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: Uint128::from(770000u128)
            }]
        })
    );
    let total_received = TOTAL_RECEIVED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_received, Uint128::from(1000000u128));
    let total_reserved = TOTAL_RESERVED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_reserved, Uint128::from(770000u128));
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_distributed, Uint128::from(230000u128));
}

#[test]
fn test_distribute_zero_to_reserve() {
    let mut deps = mock_dependencies(&[coin(1000000, DENOM)]);
    init_base_contract(deps.as_mut(), "1");
    let msg = ExecuteMsg::Distribute {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("anyone", &[]), msg);
    assert!(res.is_ok());
    let messages = res.unwrap().messages;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "distribution_contract".to_string(),
            funds: vec![Coin {
                denom: DENOM.to_string(),
                amount: Uint128::from(1000000u128)
            }],
            msg: to_binary(&DistributeMsg::Fund {}).unwrap(),
        })
    );

    let total_received = TOTAL_RECEIVED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_received, Uint128::from(1000000u128));
    let total_reserved = TOTAL_RESERVED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_reserved, Uint128::from(0u128));
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_distributed, Uint128::from(1000000u128));
}

#[test]
fn test_distribute_zero_to_distribution_contract() {
    let mut deps = mock_dependencies(&[coin(1000000, DENOM)]);
    init_base_contract(deps.as_mut(), "0");
    let msg = ExecuteMsg::Distribute {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("anyone", &[]), msg);
    assert!(res.is_ok());
    let messages = res.unwrap().messages;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "reserve_contract".to_string(),
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: Uint128::from(1000000u128)
            }]
        })
    );
    let total_received = TOTAL_RECEIVED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_received, Uint128::from(1000000u128));
    let total_reserved = TOTAL_RESERVED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_reserved, Uint128::from(1000000u128));
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.as_ref().storage).unwrap();
    assert_eq!(total_distributed, Uint128::from(0u128));
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut(), "1");
    let msg = ExecuteMsg::UpdateConfig {
        distribution_contract: None,
        reserve_contract: None,
        distribution_rate: None,
        min_period: None,
        security_dao_contract: None,
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("not_owner", &[]), msg);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});
}

#[test]
fn test_update_config_success() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut(), "1");
    let msg = ExecuteMsg::UpdateConfig {
        distribution_contract: Some("new_contract".to_string()),
        reserve_contract: Some("new_reserve_contract".to_string()),
        distribution_rate: Some(Decimal::from_str("0.11").unwrap()),
        min_period: Some(3000),
        security_dao_contract: Some("security_dao_address_contract".to_string()),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg);
    assert!(res.is_ok());
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.distribution_contract, "new_contract");
    assert_eq!(config.reserve_contract, "new_reserve_contract");
    assert_eq!(config.distribution_rate, Decimal::from_str("0.11").unwrap());
    assert_eq!(config.min_period, 3000);
    assert_eq!(
        config.security_dao_contract,
        "security_dao_address_contract"
    )
}

#[test]
fn test_update_distribution_rate_below_the_limit() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut(), "1");
    let msg = ExecuteMsg::UpdateConfig {
        distribution_contract: None,
        reserve_contract: None,
        distribution_rate: Some(Decimal::from_str("2").unwrap()),
        min_period: None,
        security_dao_contract: None,
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg);
    assert!(res.is_err());
}
