use crate::contract::{execute_request_loan, instantiate};
use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_std::testing::{message_info, mock_env};
use cosmwasm_std::{Coin, Decimal};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("addr1"), &[]);
    let instantiate_msg = InstantiateMsg {
        owner: deps.api.addr_make("neutron_dao_address").to_string(),
        source: deps.api.addr_make("neutron_dao_address").to_string(),
        fee_rate: Decimal::from_ratio(1u128, 100u128),
    };
    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
}

#[test]
fn test_request_loan_invalid_amount() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("addr1"), &[]);
    let instantiate_msg = InstantiateMsg {
        owner: deps.api.addr_make("neutron_dao_address").to_string(),
        source: deps.api.addr_make("neutron_dao_address").to_string(),
        fee_rate: Decimal::from_ratio(1u128, 100u128),
    };

    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

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
