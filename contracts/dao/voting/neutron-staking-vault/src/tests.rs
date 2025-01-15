use crate::contract::{instantiate, execute, sudo, query};
use crate::msg::{InstantiateMsg, ExecuteMsg, SudoMsg, QueryMsg};
use crate::state::{CONFIG, DAO, VALIDATORS, DELEGATIONS};
use crate::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, Uint128, Env, Decimal256};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        owner: "creator".to_string(),
        name: "Test Vault".to_string(),
        description: "Test Description".to_string(),
        denom: "uatom".to_string(),
    };
    let info = mock_info("creator", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].value, "instantiate");

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.name, "Test Vault");
    assert_eq!(config.owner, Addr::unchecked("creator"));
    assert_eq!(config.denom, "uatom");

    let dao = DAO.load(deps.as_ref().storage).unwrap();
    assert_eq!(dao, Addr::unchecked("creator"));
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies();

    let instantiate_msg = InstantiateMsg {
        owner: "creator".to_string(),
        name: "Test Vault".to_string(),
        description: "Test Description".to_string(),
        denom: "uatom".to_string(),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let update_msg = ExecuteMsg::UpdateConfig {
        owner: "new_owner".to_string(),
        name: "Updated Vault".to_string(),
        description: "Updated Description".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap();

    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].value, "update_config");

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.name, "Updated Vault");
    assert_eq!(config.owner, Addr::unchecked("new_owner"));
    assert_eq!(config.description, "Updated Description");
}

#[test]
fn test_after_validator_bonded() {
    let mut deps = mock_dependencies();

    let instantiate_msg = InstantiateMsg {
        owner: "creator".to_string(),
        name: "Test Vault".to_string(),
        description: "Test Description".to_string(),
        denom: "uatom".to_string(),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    VALIDATORS.save(
        deps.as_mut().storage,
        &Addr::unchecked("validator1"),
        &crate::state::Validator {
            address: Addr::unchecked("validator1"),
            bonded: false,
            total_tokens: Uint128::zero(),
            total_shares: Uint128::zero(),
            active: false,
        },
        0,
    )
        .unwrap();

    let sudo_msg = SudoMsg::AfterValidatorBonded {
        val_address: "validator1".to_string(),
    };

    let res = sudo(deps.as_mut(), mock_env(), sudo_msg).unwrap();
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].value, "validator_bonded");

    let validator = VALIDATORS.load(deps.as_ref().storage, &Addr::unchecked("validator1")).unwrap();
    assert!(validator.active);
}

#[test]
fn test_before_validator_slashed() {
    let mut deps = mock_dependencies();

    let instantiate_msg = InstantiateMsg {
        owner: "creator".to_string(),
        name: "Test Vault".to_string(),
        description: "Test Description".to_string(),
        denom: "uatom".to_string(),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    VALIDATORS.save(
        deps.as_mut().storage,
        &Addr::unchecked("validator1"),
        &crate::state::Validator {
            address: Addr::unchecked("validator1"),
            bonded: true,
            total_tokens: Uint128::from(100u128),
            total_shares: Uint128::from(100u128),
            active: true,
        },
        0,
    )
        .unwrap();

    let sudo_msg = SudoMsg::BeforeValidatorSlashed {
        val_address: "validator1".to_string(),
        slashing_fraction: Decimal256::percent(50),
    };

    let res = sudo(deps.as_mut(), mock_env(), sudo_msg).unwrap();
    assert_eq!(res.attributes.len(), 5);
    assert_eq!(res.attributes[0].value, "before_validator_slashed");

    let validator = VALIDATORS.load(deps.as_ref().storage, &Addr::unchecked("validator1")).unwrap();
    assert_eq!(validator.total_tokens, Uint128::from(50u128));
    assert_eq!(validator.total_shares, Uint128::from(50u128));
}

#[test]
fn test_after_delegation_modified() {
    let mut deps = mock_dependencies();

    let instantiate_msg = InstantiateMsg {
        owner: "creator".to_string(),
        name: "Test Vault".to_string(),
        description: "Test Description".to_string(),
        denom: "uatom".to_string(),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    VALIDATORS.save(
        deps.as_mut().storage,
        &Addr::unchecked("validator1"),
        &crate::state::Validator {
            address: Addr::unchecked("validator1"),
            bonded: true,
            total_tokens: Uint128::from(100u128),
            total_shares: Uint128::from(100u128),
            active: true,
        },
        0,
    )
        .unwrap();

    let sudo_msg = SudoMsg::AfterDelegationModified {
        delegator_address: "delegator1".to_string(),
        val_address: "validator1".to_string(),
    };

    let res = sudo(deps.as_mut(), mock_env(), sudo_msg);
    assert!(res.is_err()); // Mock does not simulate staking state
}