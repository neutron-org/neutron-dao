use crate::{
    contract::{execute, instantiate, query},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{CONFIG, FUND_COUNTER, PENDING_DISTRIBUTION, SHARES},
    testing::mock_querier::mock_dependencies,
};
use cosmwasm_std::{
    coin, coins, from_binary,
    testing::{mock_env, mock_info},
    Addr, BankMsg, CosmosMsg, DepsMut, Empty, Uint128,
};
use exec_control::pause::{PauseError, PauseInfoResponse};

const DENOM: &str = "denom";
const MAIN_DAO_ADDR: &str = "main_dao";
const NEW_MAIN_DAO_ADDR: &str = "new_main_dao";
const SECURITY_DAO_ADDR: &str = "security_dao";

pub fn init_base_contract(deps: DepsMut<Empty>) {
    let msg = InstantiateMsg {
        denom: DENOM.to_string(),
        main_dao_address: Addr::unchecked(MAIN_DAO_ADDR),
        security_dao_address: Addr::unchecked(SECURITY_DAO_ADDR),
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
fn test_fund_no_funds() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Fund {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("someone", &[]), msg);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::NoFundsSent {});
}

#[test]
fn test_fund_no_shares() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Fund {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("someone", &[coin(10000u128, DENOM)]),
        msg,
    );
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::NoSharesSent {});
}

#[test]
fn test_fund_success() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr1"),
            &Uint128::from(1u128),
        )
        .unwrap();
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr2"),
            &Uint128::from(3u128),
        )
        .unwrap();
    let msg = ExecuteMsg::Fund {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("someone", &[coin(10000u128, DENOM)]),
        msg,
    );
    assert!(res.is_ok());
    assert_eq!(
        PENDING_DISTRIBUTION
            .load(deps.as_ref().storage, Addr::unchecked("addr1"))
            .unwrap(),
        Uint128::from(2500u128)
    );
    assert_eq!(
        PENDING_DISTRIBUTION
            .load(deps.as_ref().storage, Addr::unchecked("addr2"))
            .unwrap(),
        Uint128::from(7500u128)
    );
    let fund_counter = FUND_COUNTER.load(deps.as_ref().storage).unwrap();
    assert_eq!(fund_counter, 1u64);
}

#[test]
fn test_fund_success_with_dust() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr1"),
            &Uint128::from(1u128),
        )
        .unwrap();
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr2"),
            &Uint128::from(3u128),
        )
        .unwrap();
    let msg = ExecuteMsg::Fund {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("someone", &[coin(10001u128, DENOM)]),
        msg,
    );
    assert!(res.is_ok());
    println!("{:?}", res.unwrap().attributes);
    assert_eq!(
        PENDING_DISTRIBUTION
            .load(deps.as_ref().storage, Addr::unchecked("addr1"))
            .unwrap(),
        Uint128::from(2501u128)
    );
    assert_eq!(
        PENDING_DISTRIBUTION
            .load(deps.as_ref().storage, Addr::unchecked("addr2"))
            .unwrap(),
        Uint128::from(7500u128)
    );
    let fund_counter = FUND_COUNTER.load(deps.as_ref().storage).unwrap();
    assert_eq!(fund_counter, 1u64);
}

#[test]
fn test_withdraw_no_pending() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("someone", &[]), msg);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::NoPendingDistribution {});
}

#[test]
fn test_withdraw_success() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    PENDING_DISTRIBUTION
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr1"),
            &Uint128::from(1000u128),
        )
        .unwrap();
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("addr1", &[]), msg);
    assert!(res.is_ok());
    // check message
    let messages = res.unwrap().messages;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr1".to_string(),
            amount: vec![coin(1000u128, DENOM)],
        })
    );
    assert_eq!(
        PENDING_DISTRIBUTION
            .may_load(deps.as_ref().storage, Addr::unchecked("addr1"))
            .unwrap(),
        None
    );
}

#[test]
fn test_set_shares_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::SetShares {
        shares: vec![("addr1".to_string(), Uint128::from(1u128))],
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("someone", &[]), msg);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});
}

#[test]
fn test_set_shares() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr1"),
            &Uint128::from(1u128),
        )
        .unwrap();
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr2"),
            &Uint128::from(3u128),
        )
        .unwrap();
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr3"),
            &Uint128::from(3u128),
        )
        .unwrap();
    let msg = ExecuteMsg::SetShares {
        shares: vec![
            ("addr1".to_string(), Uint128::from(1u128)),
            ("addr2".to_string(), Uint128::from(2u128)),
        ],
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());
    assert_eq!(
        SHARES
            .load(deps.as_ref().storage, Addr::unchecked("addr1"))
            .unwrap(),
        Uint128::from(1u128)
    );
    assert_eq!(
        SHARES
            .load(deps.as_ref().storage, Addr::unchecked("addr2"))
            .unwrap(),
        Uint128::from(2u128)
    );
    assert_eq!(
        SHARES
            .may_load(deps.as_ref().storage, Addr::unchecked("addr3"))
            .unwrap(),
        None
    );
}

#[test]
fn test_query_shares() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr1"),
            &Uint128::from(1u128),
        )
        .unwrap();
    SHARES
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr2"),
            &Uint128::from(3u128),
        )
        .unwrap();
    let msg = QueryMsg::Shares {};
    let res = query(deps.as_ref(), mock_env(), msg);
    assert!(res.is_ok());
    let value: Vec<(String, Uint128)> = from_binary(&res.unwrap()).unwrap();
    assert_eq!(
        value,
        vec![
            ("addr1".to_string(), Uint128::from(1u128)),
            ("addr2".to_string(), Uint128::from(3u128))
        ]
    );
}

#[test]
fn test_query_pending() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    PENDING_DISTRIBUTION
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr1"),
            &Uint128::from(1u128),
        )
        .unwrap();
    PENDING_DISTRIBUTION
        .save(
            deps.as_mut().storage,
            Addr::unchecked("addr2"),
            &Uint128::from(3u128),
        )
        .unwrap();
    let msg = QueryMsg::Pending {};
    let res = query(deps.as_ref(), mock_env(), msg);
    assert!(res.is_ok());
    let value: Vec<(String, Uint128)> = from_binary(&res.unwrap()).unwrap();
    assert_eq!(
        value,
        vec![
            ("addr1".to_string(), Uint128::from(1u128)),
            ("addr2".to_string(), Uint128::from(3u128))
        ]
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
