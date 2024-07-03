use crate::contract::{execute_request_loan, instantiate};
use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Coin, Decimal};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("addr1", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: "neutron_dao_address".to_string(),
            source: "neutron_dao_address".to_string(),
            fee_rate: Decimal::from_ratio(1u128, 100u128),
        },
    )
    .unwrap();
}

#[test]
fn test_request_loan_invalid_amount() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("addr1", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: "neutron_dao_address".to_string(),
            source: "neutron_dao_address".to_string(),
            fee_rate: Decimal::from_ratio(1u128, 100u128),
        },
    )
    .unwrap();

    // ------------------------------- Duplicate denoms
    let amount_duplicate_coins = vec![Coin::new(10u128, "untrn"), Coin::new(10u128, "untrn")];
    let err = execute_request_loan(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        amount_duplicate_coins,
    )
    .unwrap_err();
    assert_eq!(err, ContractError::DuplicateDenoms {});

    // ------------------------------- Zero coins
    let amount_duplicate_coins = vec![Coin::new(10u128, "untrn"), Coin::new(0u128, "uatom")];
    let err = execute_request_loan(deps.as_mut(), env, info, amount_duplicate_coins).unwrap_err();
    assert_eq!(
        err,
        ContractError::ZeroRequested {
            denom: "uatom".to_string()
        }
    )
}
