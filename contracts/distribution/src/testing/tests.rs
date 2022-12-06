use cosmwasm_std::{
    coin, coins,
    testing::{mock_env, mock_info},
    DepsMut, Empty, Uint128,
};

use crate::{
    contract::{execute, instantiate},
    msg::{ExecuteMsg, InstantiateMsg},
    state::{CONFIG, PENDING_DISTRIBUTION, SHARES},
    testing::mock_querier::mock_dependencies,
};

const DENOM: &str = "denom";

pub fn init_base_contract(deps: DepsMut<Empty>) {
    let msg = InstantiateMsg {
        denom: DENOM.to_string(),
        owner: "owner".to_string(),
        dao: "dao".to_string(),
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
fn test_fund_no_funds() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Fund {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("someone", &[]), msg);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "Generic error: no funds sent");
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
    assert_eq!(res.unwrap_err().to_string(), "Generic error: no shares set");
}

#[test]
fn test_fund_success() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    SHARES
        .save(
            deps.as_mut().storage,
            "addr1".as_bytes(),
            &Uint128::from(1u128),
        )
        .unwrap();
    SHARES
        .save(
            deps.as_mut().storage,
            "addr2".as_bytes(),
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
            .load(deps.as_ref().storage, "addr1".as_bytes())
            .unwrap(),
        Uint128::from(2500u128)
    );
    assert_eq!(
        PENDING_DISTRIBUTION
            .load(deps.as_ref().storage, "addr2".as_bytes())
            .unwrap(),
        Uint128::from(7500u128)
    );
}

#[test]
fn test_withdraw_no_pending() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("someone", &[]), msg);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Generic error: no pending distribution"
    );
}

#[test]
fn test_withdraw_success() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    PENDING_DISTRIBUTION
        .save(
            deps.as_mut().storage,
            "addr1".as_bytes(),
            &Uint128::from(1000u128),
        )
        .unwrap();
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), mock_env(), mock_info("addr1", &[]), msg);
    assert!(res.is_ok());
    assert_eq!(
        PENDING_DISTRIBUTION
            .may_load(deps.as_ref().storage, "addr1".as_bytes())
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
    assert_eq!(
        res.unwrap_err().to_string(),
        "Generic error: only dao can set shares"
    );
}

#[test]
fn test_set_shares() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    SHARES
        .save(
            deps.as_mut().storage,
            "addr1".as_bytes(),
            &Uint128::from(1u128),
        )
        .unwrap();
    SHARES
        .save(
            deps.as_mut().storage,
            "addr2".as_bytes(),
            &Uint128::from(3u128),
        )
        .unwrap();
    SHARES
        .save(
            deps.as_mut().storage,
            "addr3".as_bytes(),
            &Uint128::from(3u128),
        )
        .unwrap();
    let msg = ExecuteMsg::SetShares {
        shares: vec![
            ("addr1".to_string(), Uint128::from(1u128)),
            ("addr2".to_string(), Uint128::from(2u128)),
        ],
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("dao", &[]), msg);
    assert!(res.is_ok());
    assert_eq!(
        SHARES
            .load(deps.as_ref().storage, "addr1".as_bytes())
            .unwrap(),
        Uint128::from(1u128)
    );
    assert_eq!(
        SHARES
            .load(deps.as_ref().storage, "addr2".as_bytes())
            .unwrap(),
        Uint128::from(2u128)
    );
    assert_eq!(
        SHARES
            .may_load(deps.as_ref().storage, "addr3".as_bytes())
            .unwrap(),
        None
    );
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::UpdateConfig {
        dao: "new_dao".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("someone", &[]), msg);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Generic error: only dao can update config"
    );
}

#[test]
fn test_update_config_success() {
    let mut deps = mock_dependencies(&[]);
    init_base_contract(deps.as_mut());
    let msg = ExecuteMsg::UpdateConfig {
        dao: "new_dao".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("dao", &[]), msg);
    assert!(res.is_ok());
    assert_eq!(
        CONFIG.load(deps.as_ref().storage).unwrap().dao,
        "new_dao".to_string()
    );
}
