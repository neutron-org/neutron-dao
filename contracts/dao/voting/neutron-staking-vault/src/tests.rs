#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, Addr, Decimal256, Order, StdResult, Uint128};
    use crate::contract::{after_delegation_modified, after_validator_begin_unbonding, after_validator_bonded, after_validator_created, after_validator_removed, before_delegation_removed, before_validator_slashed, execute, instantiate};
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use crate::state::{Delegation, Validator, CONFIG, DAO, DELEGATIONS, VALIDATORS};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            name: "Test DAO".to_string(),
            description: "A test DAO contract".to_string(),
            owner: "owner".to_string(),
            denom: "denom".to_string(),
        };

        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);
        assert!(res.is_ok());

        // Validate the stored config
        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.name, "Test DAO");
        assert_eq!(config.description, "A test DAO contract");
        assert_eq!(config.owner, Addr::unchecked("owner"));
        assert_eq!(config.denom, "denom");

        // Validate DAO storage
        let dao = DAO.load(&deps.storage).unwrap();
        assert_eq!(dao, Addr::unchecked("creator"));
    }

    #[test]
    fn test_execute_update_config() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let msg = InstantiateMsg {
            name: "Test DAO".to_string(),
            description: "A test DAO contract".to_string(),
            owner: "owner".to_string(),
            denom: "denom".to_string(),
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Update config with correct owner
        let update_msg = ExecuteMsg::UpdateConfig {
            owner: "new_owner".to_string(),
            name: "Updated DAO".to_string(),
            description: "Updated description".to_string(),
        };
        let info = mock_info("owner", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, update_msg);
        assert!(res.is_ok());

        // Validate updated config
        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.name, "Updated DAO");
        assert_eq!(config.description, "Updated description");
        assert_eq!(config.owner, Addr::unchecked("new_owner"));
    }

    #[test]
    fn test_update_config_unauthorized() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let msg = InstantiateMsg {
            name: "Test DAO".to_string(),
            description: "A test DAO contract".to_string(),
            owner: "owner".to_string(),
            denom: "denom".to_string(),
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to update config with wrong owner
        let update_msg = ExecuteMsg::UpdateConfig {
            owner: "new_owner".to_string(),
            name: "Updated DAO".to_string(),
            description: "Updated description".to_string(),
        };
        let info = mock_info("unauthorized", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, update_msg);
        assert!(res.is_err());

        // Ensure config is unchanged
        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.name, "Test DAO");
        assert_eq!(config.description, "A test DAO contract");
        assert_eq!(config.owner, Addr::unchecked("owner"));
    }
    #[test]
    fn test_after_validator_bonded() {
        let mut deps = mock_dependencies();

        // Add a validator to the state
        let validator_addr = Addr::unchecked("validator1");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: false,
            total_tokens: Uint128::zero(),
            total_shares: Uint128::zero(),
            active: false,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        // Call after_validator_bonded
        let res = after_validator_bonded(
            deps.as_mut(),
            mock_env(),
            validator_addr.to_string(),
        );
        assert!(res.is_ok());

        // Check the updated validator state
        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert!(updated_validator.active);
        assert!(updated_validator.bonded);
    }

    #[test]
    fn test_before_validator_slashed_no_delegations() {
        let mut deps = mock_dependencies();

        // Add a validator with no delegations
        let validator_addr = Addr::unchecked("validator1");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        let slashing_fraction = Decimal256::percent(10); // 10% slashing
        let res = before_validator_slashed(
            deps.as_mut(),
            mock_env(),
            validator_addr.to_string(),
            slashing_fraction,
        );
        assert!(res.is_ok());

        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert_eq!(updated_validator.total_tokens, Uint128::new(900));
        assert_eq!(updated_validator.total_shares, Uint128::new(900));
    }

    #[test]
    fn test_after_validator_begin_unbonding() {
        let mut deps = mock_dependencies();

        // Add a bonded validator
        let validator_addr = Addr::unchecked("validator1");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        // Add delegations for the validator
        let delegator_addr = Addr::unchecked("delegator1");
        let delegation = Delegation {
            delegator_address: delegator_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(500),
        };
        DELEGATIONS.save(
            deps.as_mut().storage,
            (&delegator_addr, &validator_addr),
            &delegation,
            0,
        )
            .unwrap();

        // Call after_validator_begin_unbonding
        let res = after_validator_begin_unbonding(deps.as_mut(), mock_env(), validator_addr.to_string());
        assert!(res.is_ok());

        // Check the updated validator state
        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert!(!updated_validator.bonded);

        // Check that delegations remain unchanged FOR NOW
        let updated_delegation = DELEGATIONS
            .load(deps.as_ref().storage, (&delegator_addr, &validator_addr))
            .unwrap();
        assert_eq!(updated_delegation.shares, Uint128::new(500));
    }


    #[test]
    fn test_before_validator_slashed() {
        let mut deps = mock_dependencies();

        // Add a validator to the state
        let validator_addr = Addr::unchecked("validator1");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        // Add delegations to the state
        let delegator_addr = Addr::unchecked("delegator1");
        let delegation = Delegation {
            delegator_address: delegator_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(500),
        };
        DELEGATIONS.save(
            deps.as_mut().storage,
            (&delegator_addr, &validator_addr),
            &delegation,
            0,
        )
            .unwrap();

        let slashing_fraction = Decimal256::percent(10); // 10% slashing
        let res = before_validator_slashed(
            deps.as_mut(),
            mock_env(),
            validator_addr.to_string(),
            slashing_fraction,
        );
        assert!(res.is_ok(), "Error: {:?}", res.err());

        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert_eq!(updated_validator.total_tokens, Uint128::new(900));
        assert_eq!(updated_validator.total_shares, Uint128::new(900));

        let updated_delegation = DELEGATIONS
            .load(deps.as_ref().storage, (&delegator_addr, &validator_addr))
            .unwrap();
        assert_eq!(updated_delegation.shares, Uint128::new(450));
    }

    #[test]
    fn test_after_validator_begin_unbonding_no_delegations() {
        let mut deps = mock_dependencies();

        // Add a bonded validator with no delegations
        let validator_addr = Addr::unchecked("validator3");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(3000),
            total_shares: Uint128::new(3000),
            active: true,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        let res = after_validator_begin_unbonding(deps.as_mut(), mock_env(), validator_addr.to_string());
        assert!(res.is_ok());

        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert!(!updated_validator.bonded);
        assert_eq!(updated_validator.total_tokens, Uint128::new(3000));
        assert_eq!(updated_validator.total_shares, Uint128::new(3000));

        let delegations = DELEGATIONS
            .prefix(&validator_addr)
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>();
        assert!(delegations.unwrap().is_empty());
    }

    #[test]
    fn test_after_validator_created() {
        let mut deps = mock_dependencies();

        let validator_addr = Addr::unchecked("validator4");
        let res = after_validator_created(deps.as_mut(), mock_env(), validator_addr.to_string());
        assert!(res.is_ok());

        let validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert_eq!(validator.address, validator_addr);
        assert!(!validator.bonded);
        assert_eq!(validator.total_tokens, Uint128::zero());
        assert_eq!(validator.total_shares, Uint128::zero());
        assert!(validator.active);
    }

    #[test]
    fn test_before_delegation_removed() {
        let mut deps = mock_dependencies();

        let validator_addr = Addr::unchecked("validator5");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        let delegator_addr = Addr::unchecked("delegator3");
        let delegation = Delegation {
            delegator_address: delegator_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(500),
        };
        DELEGATIONS.save(
            deps.as_mut().storage,
            (&delegator_addr, &validator_addr),
            &delegation,
            0,
        )
            .unwrap();

        let res = before_delegation_removed(
            deps.as_mut(),
            mock_env(),
            delegator_addr.to_string(),
            validator_addr.to_string(),
        );
        assert!(res.is_ok());

        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert_eq!(updated_validator.total_tokens, Uint128::new(500));
        assert_eq!(updated_validator.total_shares, Uint128::new(500));

        let updated_delegation = DELEGATIONS
            .may_load(deps.as_ref().storage, (&delegator_addr, &validator_addr))
            .unwrap();
        assert!(updated_delegation.is_some());
        assert_eq!(updated_delegation.unwrap().shares, Uint128::zero());
    }

    #[test]
    fn test_after_validator_bonded() {
        let mut deps = mock_dependencies();

        let validator_addr = Addr::unchecked("validator7");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: false,
            total_tokens: Uint128::zero(),
            total_shares: Uint128::zero(),
            active: true,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        let res = after_validator_bonded(deps.as_mut(), mock_env(), validator_addr.to_string());
        assert!(res.is_ok());

        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert!(updated_validator.bonded);
        assert!(updated_validator.total_tokens > Uint128::zero());
        assert!(updated_validator.total_shares > Uint128::zero());
    }

    #[test]
    fn test_after_delegation_modified() {
        // let mut deps = mock_dependencies();
        //
        // // Add a validator
        // let validator_addr = Addr::unchecked("validator8");
        // let validator = Validator {
        //     address: validator_addr.clone(),
        //     bonded: true,
        //     total_tokens: Uint128::new(1000),
        //     total_shares: Uint128::new(1000),
        //     active: true,
        // };
        // VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();
        //
        // // Add an existing delegation
        // let delegator_addr = Addr::unchecked("delegator5");
        // let delegation = Delegation {
        //     delegator_address: delegator_addr.clone(),
        //     validator_address: validator_addr.clone(),
        //     shares: Uint128::new(500),
        // };
        // DELEGATIONS.save(
        //     deps.as_mut().storage,
        //     (&delegator_addr, &validator_addr),
        //     &delegation,
        //     0,
        // )
        //     .unwrap();
        //
        // // Simulate delegation modification
        // let new_delegation_shares = Uint128::new(700); // Increased shares
        // let mut querier = mock_dependencies().querier;
        // querier.update_staking(validator_addr.to_string(), vec![delegator_addr.to_string()], new_delegation_shares);
        //
        // // Call after_delegation_modified
        // let res = after_delegation_modified(
        //     deps.as_mut(),
        //     mock_env(),
        //     delegator_addr.to_string(),
        //     validator_addr.to_string(),
        // );
        // assert!(res.is_ok());
        //
        // // Check the updated validator state
        // let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        // assert_eq!(updated_validator.total_shares, Uint128::new(1200));
        // assert_eq!(updated_validator.total_tokens, Uint128::new(1200));
        //
        // // Check the updated delegation
        // let updated_delegation = DELEGATIONS
        //     .load(deps.as_ref().storage, (&delegator_addr, &validator_addr))
        //     .unwrap();
        // assert_eq!(updated_delegation.shares, Uint128::new(700));
    }


    #[test]
    fn test_before_delegation_removed() {
        let mut deps = mock_dependencies();

        let validator_addr = Addr::unchecked("validator9");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        let delegator_addr = Addr::unchecked("delegator6");
        let delegation = Delegation {
            delegator_address: delegator_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(400),
        };
        DELEGATIONS.save(
            deps.as_mut().storage,
            (&delegator_addr, &validator_addr),
            &delegation,
            0,
        )
            .unwrap();

        let res = before_delegation_removed(
            deps.as_mut(),
            mock_env(),
            delegator_addr.to_string(),
            validator_addr.to_string(),
        );
        assert!(res.is_ok());

        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert_eq!(updated_validator.total_shares, Uint128::new(600));
        assert_eq!(updated_validator.total_tokens, Uint128::new(600));

        let updated_delegation = DELEGATIONS
            .load(deps.as_ref().storage, (&delegator_addr, &validator_addr))
            .unwrap();
        assert_eq!(updated_delegation.shares, Uint128::zero());
    }

    #[test]
    fn test_after_validator_created() {
        let mut deps = mock_dependencies();

        let validator_addr = Addr::unchecked("validator10");

        let res = after_validator_created(deps.as_mut(), mock_env(), validator_addr.to_string());
        assert!(res.is_ok());

        let validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert_eq!(validator.address, validator_addr);
        assert!(!validator.bonded);
        assert_eq!(validator.total_tokens, Uint128::zero());
        assert_eq!(validator.total_shares, Uint128::zero());
        assert!(validator.active);
    }

    #[test]
    fn test_after_validator_removed() {
        let mut deps = mock_dependencies();

        let validator_addr = Addr::unchecked("validator11");
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS.save(deps.as_mut().storage, &validator_addr, &validator, 0).unwrap();

        // Call after_validator_removed
        let res = after_validator_removed(
            deps.as_mut(),
            mock_env(),
            "valcons_address".to_string(),
            validator_addr.to_string(),
        );
        assert!(res.is_ok());

        let validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert!(!validator.active);
        assert_eq!(validator.bonded, true);
        assert_eq!(validator.total_tokens, Uint128::new(1000));
        assert_eq!(validator.total_shares, Uint128::new(1000));
    }

}
