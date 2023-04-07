use crate::{
    contract::{execute, instantiate, query},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::CONFIG,
    testing::mock_querier::mock_dependencies,
};
use cosmwasm_std::{
    coin, coins, from_binary,
    testing::{mock_env, mock_info},
    BankMsg, Coin, CosmosMsg, DepsMut, Empty, Uint128,
};
use exec_control::pause::{PauseError, PauseInfoResponse};

const DENOM: &str = "denom";
const MAIN_DAO_ADDR: &str = "main_dao";
const NEW_MAIN_DAO_ADDR: &str = "new_main_dao";
const SECURITY_DAO_ADDR: &str = "security_dao";

pub fn init_base_contract(deps: DepsMut<Empty>) {
    let msg = InstantiateMsg {
        denom: DENOM.to_string(),
        main_dao_address: MAIN_DAO_ADDR.to_string(),
        security_dao_address: SECURITY_DAO_ADDR.to_string(),
    };
    let info = mock_info("creator", &coins(2, DENOM));
    instantiate(deps, mock_env(), info, msg).unwrap();
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::TransferOwnership(NEW_MAIN_DAO_ADDR.to_string());
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config.main_dao_address.to_string(),
        NEW_MAIN_DAO_ADDR.to_string()
    );
}

#[test]
fn test_payout_no_money() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Payout {
        amount: Uint128::from(500000u128),
        recipient: "some".to_string(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::InsufficientFunds {});
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
    assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});
}

#[test]
fn test_payout_success() {
    let mut deps = mock_dependencies(&[coin(1000000, DENOM)]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Payout {
        amount: Uint128::from(400000u128),
        recipient: "some".to_string(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
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

#[test]
fn test_pause() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());

    // pause contracts for 10 blocks from main dao
    let msg = ExecuteMsg::Pause { duration: 10u64 };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());
    let pause_info: PauseInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::PauseInfo {}).unwrap()).unwrap();
    assert_eq!(
        pause_info,
        PauseInfoResponse::Paused {
            until_height: mock_env().block.height + 10
        }
    );

    // security dao can't unpause contracts
    let msg = ExecuteMsg::Unpause {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(SECURITY_DAO_ADDR, &[]),
        msg,
    );
    assert_eq!(
        res.err().unwrap(),
        ContractError::PauseError(PauseError::Unauthorized {})
    );

    // unable to execute anything
    let msg = ExecuteMsg::TransferOwnership(NEW_MAIN_DAO_ADDR.to_string());
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert_eq!(
        res.err().unwrap(),
        ContractError::PauseError(PauseError::Paused {})
    );

    let mut env = mock_env();
    env.block.height += 11;

    // but we can do it after 11 blocks
    let msg = ExecuteMsg::TransferOwnership(NEW_MAIN_DAO_ADDR.to_string());
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());

    env.block.height += 15;

    // pause contracts for 10 blocks from security dao
    let msg = ExecuteMsg::Pause { duration: 10u64 };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(SECURITY_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());
    let pause_info: PauseInfoResponse =
        from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::PauseInfo {}).unwrap()).unwrap();
    assert_eq!(
        pause_info,
        PauseInfoResponse::Paused {
            until_height: env.block.height + 10
        }
    );

    // only main dao can unpause contracts
    let msg = ExecuteMsg::Unpause {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(NEW_MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok(),);
    let pause_info: PauseInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::PauseInfo {}).unwrap()).unwrap();
    assert_eq!(pause_info, PauseInfoResponse::Unpaused {});
}
