use cosmwasm_std::Uint128;
use crate::state::Validator;
#[cfg(test)]

mod tests {
    use super::*;
    use cosmwasm_std::{from_binary, testing::{mock_dependencies, mock_env, mock_info}, Addr, Decimal256, Uint128};
    use cwd_interface::voting::TotalPowerAtHeightResponse;
    use crate::contract::{after_delegation_modified, after_validator_begin_unbonding, after_validator_bonded, after_validator_created, before_delegation_removed, before_validator_slashed, execute, get_delegations_filtered_by_validator, instantiate, query, query_total_power_at_height, query_voting_power_at_height};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{Config, Delegation, Validator, BLACKLISTED_ADDRESSES, CONFIG, DAO, DELEGATIONS, VALIDATORS};

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
    fn test_add_and_remove_from_blacklist() {
        let mut deps = mock_dependencies();

        // Initialize config with owner
        let config = Config {
            name: String::from("Test Config"),
            description: String::from("Testing blacklist functionality"),
            owner: Addr::unchecked("admin"),
            denom: String::from("testdenom"),
        };
        CONFIG.save(deps.as_mut().storage, &config).unwrap();

        // Add addresses to the blacklist
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::AddToBlacklist {
                addresses: vec![String::from(Addr::unchecked("addr1")), String::from(Addr::unchecked("addr2"))],
            },
        );
        assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

        // Verify that addresses are blacklisted
        let is_addr1_blacklisted = BLACKLISTED_ADDRESSES.load(deps.as_ref().storage, Addr::unchecked("addr1")).unwrap_or(false);
        let is_addr2_blacklisted = BLACKLISTED_ADDRESSES.load(deps.as_ref().storage, Addr::unchecked("addr2")).unwrap_or(false);
        assert!(is_addr1_blacklisted, "Address addr1 is not blacklisted");
        assert!(is_addr2_blacklisted, "Address addr2 is not blacklisted");

        // Remove addresses from the blacklist
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::RemoveFromBlacklist {
                addresses: vec![String::from(Addr::unchecked("addr1")), String::from(Addr::unchecked("addr2"))],
            },
        );
        assert!(res.is_ok(), "Error removing from blacklist: {:?}", res.err());

        // Verify that addresses are no longer blacklisted
        let is_addr1_blacklisted = BLACKLISTED_ADDRESSES.may_load(deps.as_ref().storage, Addr::unchecked("addr1")).unwrap();
        let is_addr2_blacklisted = BLACKLISTED_ADDRESSES.may_load(deps.as_ref().storage, Addr::unchecked("addr2")).unwrap();
        assert!(is_addr1_blacklisted.is_none(), "Address addr1 is still blacklisted");
        assert!(is_addr2_blacklisted.is_none(), "Address addr2 is still blacklisted");
    }

    #[test]
    fn test_check_if_address_is_blacklisted() {
        let mut deps = mock_dependencies();

        // Initialize config with owner
        let config = Config {
            name: String::from("Test Config"),
            description: String::from("Testing blacklist functionality"),
            owner: Addr::unchecked("admin"),
            denom: String::from("testdenom"),
        };
        CONFIG.save(deps.as_mut().storage, &config).unwrap();

        // Add an address to the blacklist
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::AddToBlacklist {
                addresses: vec![String::from(Addr::unchecked("addr1"))],
            },
        );
        assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

        // Query if the address is blacklisted
        let query_res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsAddressBlacklisted {
                address: "addr1".to_string(),
            },
        );
        assert!(query_res.is_ok(), "Error querying blacklist status: {:?}", query_res.err());

        let is_blacklisted: bool = from_binary(&query_res.unwrap()).unwrap();
        assert!(is_blacklisted, "Address addr1 should be blacklisted");

        // Query an address that is not blacklisted
        let query_res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsAddressBlacklisted {
                address: "addr2".to_string(),
            },
        );
        assert!(query_res.is_ok(), "Error querying blacklist status: {:?}", query_res.err());

        let is_blacklisted: bool = from_binary(&query_res.unwrap()).unwrap();
        assert!(!is_blacklisted, "Address addr2 should not be blacklisted");
    }

    #[test]
    fn test_total_vp_excludes_blacklisted_addresses() {
        let mut deps = mock_dependencies();

        // Add validators
        let validator1 = Validator {
            address: Addr::unchecked("validator1"),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS
            .save(deps.as_mut().storage, &Addr::unchecked("validator1"), &validator1, 0)
            .unwrap();

        let validator2 = Validator {
            address: Addr::unchecked("validator2"),
            bonded: true,
            total_tokens: Uint128::new(500),
            total_shares: Uint128::new(500),
            active: true,
        };
        VALIDATORS
            .save(deps.as_mut().storage, &Addr::unchecked("validator2"), &validator2, 0)
            .unwrap();

        // Add delegations
        let delegation1 = Delegation {
            delegator_address: Addr::unchecked("addr1"),
            validator_address: Addr::unchecked("validator1"),
            shares: Uint128::new(500),
        };
        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&Addr::unchecked("addr1"), &Addr::unchecked("validator1")),
                &delegation1,
                0,
            )
            .unwrap();

        let delegation2 = Delegation {
            delegator_address: Addr::unchecked("addr2"),
            validator_address: Addr::unchecked("validator2"),
            shares: Uint128::new(500),
        };
        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&Addr::unchecked("addr2"), &Addr::unchecked("validator2")),
                &delegation2,
                0,
            )
            .unwrap();

        // Add addr2 to blacklist
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::AddToBlacklist {
                addresses: vec![String::from(Addr::unchecked("addr2"))],
            },
        );
        assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

        // Query total power at current height
        let query_res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TotalPowerAtHeight { height: None },
        );
        assert!(query_res.is_ok(), "Error querying total power: {:?}", query_res.err());

        let total_power: TotalPowerAtHeightResponse = from_binary(&query_res.unwrap()).unwrap();
        assert_eq!(
            total_power.power,
            Uint128::new(1000),
            "Total power should exclude blacklisted address"
        );
    }





    #[test]
    fn test_after_validator_bonded_with_mock_query() {
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
        VALIDATORS
            .save(deps.as_mut().storage, &validator_addr, &validator, 0)
            .unwrap();

        // // Mock the `validator` query to return expected data
        // let validator_info = QueryValidatorResponse {
        //     validator: Some(neutron_std::types::cosmos::staking::v1beta1::Validator {
        //         operator_address: validator_addr.to_string(),
        //         consensus_pubkey: None,
        //         jailed: false,
        //         status: 3, // Bonded status
        //         tokens: "1000".to_string(),
        //         delegator_shares: "1000".to_string(),
        //         description: None,
        //         unbonding_height: 0,
        //         unbonding_time: None,
        //         commission: None,
        //         min_self_delegation: None,
        //         unbonding_on_hold_ref_count: 0,
        //         unbonding_ids: vec![],
        //     }),
        // };
        //
        // deps.querier.with_custom_handler(|query| {
        //     match query {
        //         cosmwasm_std::QueryRequest::Custom(neutron_std::StakingQuery::Validator { address }) => {
        //             assert_eq!(address, validator_addr.to_string());
        //             Ok(cosmwasm_std::to_binary(&validator_info))
        //         }
        //         _ => panic!("Unexpected query: {:?}", query),
        //     }
        // });

        // Call after_validator_bonded
        let res = after_validator_bonded(deps.as_mut(), mock_env(), validator_addr.to_string());
        assert!(res.is_ok());

        // Validate the updated validator state
        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert!(updated_validator.active);
        assert!(!updated_validator.bonded); // The logic for setting bonded might need confirmation
        assert_eq!(updated_validator.total_tokens, Uint128::new(1000));
        assert_eq!(updated_validator.total_shares, Uint128::new(1000));

        // Validate the response attributes
        let response = res.unwrap();
        assert_eq!(
            response.attributes,
            vec![
                ("action", "validator_bonded"),
                ("validator_address", &*validator_addr.to_string()),
                ("total_tokens", "1000"),
                ("total_shares", "1000"),
            ]
        );
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
            total_tokens: Uint128::new(500),
            total_shares: Uint128::new(500),
            active: true,
        };
        VALIDATORS
            .save(deps.as_mut().storage, &validator_addr, &validator, 0)
            .unwrap();

        // Add delegations to the state
        let delegator_addr = Addr::unchecked("delegator1");
        let delegation = Delegation {
            delegator_address: delegator_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(500),
        };
        DELEGATIONS
            .save(
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
        assert_eq!(updated_validator.total_tokens, Uint128::new(450));
        assert_eq!(updated_validator.total_shares, Uint128::new(450));

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
        assert!(res.is_err());
    }

    #[test]
    fn test_before_validator_slashed_with_multiple_delegations() {
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
        VALIDATORS
            .save(deps.as_mut().storage, &validator_addr, &validator, 0)
            .unwrap();

        // Add multiple delegations to the state
        let delegator1_addr = Addr::unchecked("delegator1");
        let delegation1 = Delegation {
            delegator_address: delegator1_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(400),
        };
        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&delegator1_addr, &validator_addr),
                &delegation1,
                0,
            )
            .unwrap();

        let delegator2_addr = Addr::unchecked("delegator2");
        let delegation2 = Delegation {
            delegator_address: delegator2_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(600),
        };
        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&delegator2_addr, &validator_addr),
                &delegation2,
                0,
            )
            .unwrap();

        // Perform a 10% slashing
        let slashing_fraction = Decimal256::percent(10); // 10% slashing
        let res = before_validator_slashed(
            deps.as_mut(),
            mock_env(),
            validator_addr.to_string(),
            slashing_fraction,
        );
        assert!(res.is_ok(), "Error: {:?}", res.err());

        let updated_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        assert_eq!(updated_validator.total_tokens, Uint128::new(900)); // 10% of 1000 slashed
        assert_eq!(updated_validator.total_shares, Uint128::new(900));

        let updated_delegation1 = DELEGATIONS
            .load(deps.as_ref().storage, (&delegator1_addr, &validator_addr))
            .unwrap();
        assert_eq!(updated_delegation1.shares, Uint128::new(360)); // 10% of 400 slashed

        let updated_delegation2 = DELEGATIONS
            .load(deps.as_ref().storage, (&delegator2_addr, &validator_addr))
            .unwrap();
        assert_eq!(updated_delegation2.shares, Uint128::new(540)); // 10% of 600 slashed

        // Ensure validator shares match the sum of all delegations
        let total_delegation_shares = updated_delegation1.shares + updated_delegation2.shares;
        assert_eq!(
            updated_validator.total_shares, total_delegation_shares,
            "Validator total shares do not match the sum of all delegations!"
        );

        // Ensure validator tokens match the sum of all delegations
        let total_delegation_tokens = updated_delegation1.shares + updated_delegation2.shares;
        assert_eq!(
            updated_validator.total_tokens, total_delegation_tokens,
            "Validator total tokens do not match the sum of all delegations!"
        );
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
    fn test_create_delegation_and_query_voting_power() {
        // let mut deps = mock_dependencies();
        // let validator_addr = Addr::unchecked("validator1");
        // let delegator_addr = Addr::unchecked("delegator1");
        //
        // after_validator_created(deps.as_mut(), mock_env(), validator_addr.to_string()).unwrap();
        // after_validator_bonded(deps.as_mut(), mock_env(), validator_addr.to_string()).unwrap();
        //
        // let bonded_validator = VALIDATORS.load(deps.as_ref().storage, &validator_addr).unwrap();
        // assert!(bonded_validator.bonded);
        //
        // let res = after_delegation_modified(
        //     deps.as_mut(),
        //     mock_env(),
        //     delegator_addr.to_string(),
        //     validator_addr.to_string(),
        // );
        // assert!(res.is_ok());
        //
        // let delegation = DELEGATIONS
        //     .load(deps.as_ref().storage, (&delegator_addr, &validator_addr))
        //     .unwrap();
        // assert_eq!(delegation.delegator_address, delegator_addr);
        // assert_eq!(delegation.validator_address, validator_addr);
        // assert!(delegation.shares > Uint128::zero());
        //
        // let env = mock_env();
        // let query_response = query_voting_power_at_height(
        //     deps.as_ref(),
        //     env.clone(),
        //     delegator_addr.to_string(),
        //     None,
        // );
        // assert!(query_response.is_ok());
        //
        // let query_res = query_response.unwrap();
        // assert_eq!(query_res.power, delegation.shares);
        // assert_eq!(query_res.height, env.block.height);
        //
        // let total_power_res = query_total_power_at_height(deps.as_ref(), env.clone(), None);
        // assert!(total_power_res.is_ok());
        //
        // let total_power_response = total_power_res.unwrap();
        // assert_eq!(total_power_response.power, bonded_validator.total_tokens);
        // assert_eq!(total_power_response.height, env.block.height);
    }

    #[test]
    fn test_create_delegation_and_query_voting_power_direct_write() {
        let mut deps = mock_dependencies();

        let validator_addr = Addr::unchecked("validator1");
        let delegator_addr = Addr::unchecked("delegator1");

        // Write validator directly to storage
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS
            .save(deps.as_mut().storage, &validator_addr, &validator, 10)
            .unwrap();

        // Write delegation directly to storage
        let delegation = Delegation {
            delegator_address: delegator_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(500),
        };
        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&delegator_addr, &validator_addr),
                &delegation,
                10,
            )
            .unwrap();

        // Query current voting power
        let env = mock_env();
        let query_response = query_voting_power_at_height(
            deps.as_ref(),
            env.clone(),
            delegator_addr.to_string(),
            None,
        );
        assert!(query_response.is_ok());

        let query_res = query_response.unwrap();
        assert_eq!(query_res.power, delegation.shares);
        assert_eq!(query_res.height, env.block.height);

        // Query total power at current height
        let total_power_res = query_total_power_at_height(deps.as_ref(), env.clone(), None);
        assert!(total_power_res.is_ok());

        let total_power_response = total_power_res.unwrap();
        assert_eq!(total_power_response.power, validator.total_tokens);
        assert_eq!(total_power_response.height, env.block.height);

        // Query voting power at historical height
        let historical_height = 11;
        let historical_vp_res = query_voting_power_at_height(
            deps.as_ref(),
            env.clone(),
            delegator_addr.to_string(),
            Some(historical_height),
        );
        assert!(historical_vp_res.is_ok());

        let historical_vp = historical_vp_res.unwrap();
        assert_eq!(historical_vp.power, delegation.shares);
        assert_eq!(historical_vp.height, historical_height);

        // Query total power at historical height
        let historical_total_power_res =
            query_total_power_at_height(deps.as_ref(), env.clone(), Some(historical_height));
        assert!(historical_total_power_res.is_ok());

        let historical_total_power = historical_total_power_res.unwrap();
        assert_eq!(historical_total_power.power, validator.total_tokens);
        assert_eq!(historical_total_power.height, historical_height);
    }


    #[test]
    fn test_undelegation_and_query_voting_power() {
        let mut deps = mock_dependencies();

        let validator_addr = Addr::unchecked("validator1");
        let delegator_addr = Addr::unchecked("delegator1");

        // Write validator directly to storage
        let validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(1000),
            total_shares: Uint128::new(1000),
            active: true,
        };
        VALIDATORS
            .save(deps.as_mut().storage, &validator_addr, &validator, 9)
            .unwrap();

        // Write initial delegation directly to storage
        let delegation = Delegation {
            delegator_address: delegator_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(500),
        };
        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&delegator_addr, &validator_addr),
                &delegation,
                9,
            )
            .unwrap();

        // Simulate an undelegation by reducing shares and updating the state at block height 15
        let updated_delegation = Delegation {
            delegator_address: delegator_addr.clone(),
            validator_address: validator_addr.clone(),
            shares: Uint128::new(200), // Reduced shares
        };
        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&delegator_addr, &validator_addr),
                &updated_delegation,
                14,
            )
            .unwrap();

        let updated_validator = Validator {
            address: validator_addr.clone(),
            bonded: true,
            total_tokens: Uint128::new(700), // Reduced total tokens
            total_shares: Uint128::new(700), // Reduced total shares
            active: true,
        };
        VALIDATORS
            .save(deps.as_mut().storage, &validator_addr, &updated_validator, 14)
            .unwrap();

        // Query voting power before undelegation
        let env = mock_env();
        let query_vp_before = query_voting_power_at_height(
            deps.as_ref(),
            env.clone(),
            delegator_addr.to_string(),
            Some(10),
        );
        assert!(query_vp_before.is_ok());

        let vp_before = query_vp_before.unwrap();
        assert_eq!(vp_before.power, delegation.shares);
        assert_eq!(vp_before.height, 10);

        // Query voting power after undelegation
        let query_vp_after = query_voting_power_at_height(
            deps.as_ref(),
            env.clone(),
            delegator_addr.to_string(),
            Some(15),
        );
        assert!(query_vp_after.is_ok());

        let vp_after = query_vp_after.unwrap();
        assert_eq!(vp_after.power, updated_delegation.shares);
        assert_eq!(vp_after.height, 15);

        // Query total power before undelegation
        let total_power_before = query_total_power_at_height(deps.as_ref(), env.clone(), Some(10));
        assert!(total_power_before.is_ok());

        let total_power_before_res = total_power_before.unwrap();
        assert_eq!(total_power_before_res.power, validator.total_tokens);
        assert_eq!(total_power_before_res.height, 10);

        // Query total power after undelegation
        let total_power_after = query_total_power_at_height(deps.as_ref(), env.clone(), Some(15));
        assert!(total_power_after.is_ok());

        let total_power_after_res = total_power_after.unwrap();
        assert_eq!(total_power_after_res.power, updated_validator.total_tokens);
        assert_eq!(total_power_after_res.height, 15);
    }

}
