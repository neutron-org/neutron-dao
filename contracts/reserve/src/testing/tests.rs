use cosmwasm_std::{
    coin, coins,
    testing::{mock_env, mock_info},
    BankMsg, Coin, CosmosMsg, DepsMut, Empty, Uint128,
};

use crate::{
    contract::{execute, instantiate},
    msg::{ExecuteMsg, InstantiateMsg},
    state::CONFIG,
    testing::mock_querier::mock_dependencies,
};

const DENOM: &str = "denom";

pub fn init_base_contract(deps: DepsMut<Empty>) {
    let msg = InstantiateMsg {
        denom: DENOM.to_string(),

        owner: "owner".to_string(),
    };
    let info = mock_info("creator", &coins(2, DENOM));
    instantiate(deps, mock_env(), info, msg).unwrap();
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::TransferOwnership("new_owner".to_string());
    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg);
    assert!(res.is_ok());
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.owner.to_string(), "new_owner".to_string());
}

#[test]
fn test_payout_no_money() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Payout {
        amount: Uint128::from(500000u128),
        recipient: "some".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Generic error: insufficient funds"
    );
}

#[test]
fn test_payout_not_owner() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Payout {
        amount: Uint128::from(500000u128),
        recipient: "some".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("not_owner", &[]), msg);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "Generic error: unauthorized");
}

#[test]
fn test_payout_success() {
    let mut deps = mock_dependencies(&[coin(1000000, DENOM)]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Payout {
        amount: Uint128::from(400000u128),
        recipient: "some".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg);
    assert!(res.is_ok());
    let messages = res.unwrap().messages;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "some".to_string(),
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: Uint128::from(400000u128)
            }],
        })
    );
}
