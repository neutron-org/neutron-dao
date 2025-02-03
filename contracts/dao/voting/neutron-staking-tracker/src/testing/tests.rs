#[cfg(test)]
use crate::contract::{
    after_delegation_modified, after_validator_bonded, after_validator_created,
    before_delegation_removed, before_validator_slashed, execute, instantiate, query,
    query_total_power_at_height, query_voting_power_at_height,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, Delegation, Validator, BLACKLISTED_ADDRESSES, CONFIG, DAO, DELEGATIONS, VALIDATORS,
};
use crate::testing::mock_querier::mock_dependencies as dependencies;
use cosmwasm_std::testing::message_info;
use cosmwasm_std::{
    from_json,
    testing::{mock_dependencies, mock_env},
    to_json_binary, Addr, Decimal256, Uint128,
};
use neutron_std::types::cosmos::staking::v1beta1::{
    QueryValidatorResponse, Validator as CosmosValidator,
};
use std::collections::HashMap;

#[test]
fn test_query_validator_response_serialization() {
    let validator = CosmosValidator {
        operator_address: "validator1".to_string(),
        consensus_pubkey: None,
        jailed: false,
        status: 3, // Bonded
        tokens: "1000".to_string(),
        delegator_shares: "1000".to_string(),
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };

    let response = QueryValidatorResponse {
        validator: Some(validator),
    };

    let binary = to_json_binary(&response).unwrap();
    assert!(binary.len() > 0); // Ensure serialization is successful
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("neutron");

    //  Use a properly formatted Neutron Bech32 address
    let valid_neutron_address = deps
        .api
        .addr_make("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq96d9h3");

    let msg = InstantiateMsg {
        name: "Test DAO".to_string(),
        description: "A test DAO contract".to_string(),
        owner: valid_neutron_address.to_string(), //  Use valid address
        denom: "denom".to_string(),
    };

    let info = message_info(&valid_neutron_address, &[]); //  Use the same address

    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);
    if let Err(err) = &res {
        println!("Instantiation Error: {:?}", err);
    }
    assert!(res.is_ok(), "Error: {:?}", res.err());

    // Validate the stored config
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.name, "Test DAO");
    assert_eq!(config.description, "A test DAO contract");
    assert_eq!(config.owner, valid_neutron_address); //  Fix assertion
    assert_eq!(config.denom, "denom");

    // Validate DAO storage
    let dao = DAO.load(&deps.storage).unwrap();
    assert_eq!(dao, valid_neutron_address); //  Fix assertion
}

#[test]
fn test_execute_update_config() {
    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let owner = &deps.api.addr_make("owner");
    let creator = &deps.api.addr_make("creator");

    // Initialize the contract
    let msg = InstantiateMsg {
        name: "Test".to_string(),
        description: "A test contract".to_string(),
        owner: owner.to_string(),
        denom: "denom".to_string(),
    };
    let info = message_info(creator, &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let new_owner = deps.api.addr_make("new_owner");
    // Update config with correct owner
    let update_msg = ExecuteMsg::UpdateConfig {
        owner: new_owner.to_string(),
        name: "Updated DAO".to_string(),
        description: "Updated description".to_string(),
    };
    let info = message_info(owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_msg);
    assert!(res.is_ok());

    // Validate updated config
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.name, "Updated DAO");
    assert_eq!(config.description, "Updated description");
    assert_eq!(config.owner, new_owner);
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let owner = &deps.api.addr_make("owner");
    let new_owner = &deps.api.addr_make("new_owner");
    let creator = &deps.api.addr_make("creator");
    let unauthorized = &deps.api.addr_make("unauthorized");

    // Initialize the contract
    let msg = InstantiateMsg {
        name: "Test DAO".to_string(),
        description: "A test DAO contract".to_string(),
        owner: owner.to_string(),
        denom: "denom".to_string(),
    };
    let info = message_info(creator, &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Try to update config with wrong owner
    let update_msg = ExecuteMsg::UpdateConfig {
        owner: new_owner.to_string(),
        name: "Updated DAO".to_string(),
        description: "Updated description".to_string(),
    };
    let info = message_info(unauthorized, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_msg);
    assert!(res.is_err());

    // Ensure config is unchanged
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.name, "Test DAO");
    assert_eq!(config.description, "A test DAO contract");
    assert_eq!(config.owner, owner);
}

#[test]
fn test_add_and_remove_from_blacklist() {
    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let admin = deps.api.addr_make("admin");
    let addr1 = deps.api.addr_make("addr1");
    let addr2 = deps.api.addr_make("addr2");

    // Initialize config with owner
    let config = Config {
        name: String::from("Test Config"),
        description: String::from("Testing blacklist functionality"),
        owner: admin.clone(),
        denom: String::from("testdenom"),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Add addresses to the blacklist
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&admin, &[]),
        ExecuteMsg::AddToBlacklist {
            addresses: vec![String::from(addr1.clone()), String::from(addr2.clone())],
        },
    );
    assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

    // Verify that addresses are blacklisted
    let is_addr1_blacklisted = BLACKLISTED_ADDRESSES
        .load(deps.as_ref().storage, addr1.clone())
        .unwrap_or(false);
    let is_addr2_blacklisted = BLACKLISTED_ADDRESSES
        .load(deps.as_ref().storage, addr2.clone())
        .unwrap_or(false);
    assert!(is_addr1_blacklisted, "Address addr1 is not blacklisted");
    assert!(is_addr2_blacklisted, "Address addr2 is not blacklisted");

    // Remove addresses from the blacklist
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&admin, &[]),
        ExecuteMsg::RemoveFromBlacklist {
            addresses: vec![String::from(addr1.clone()), String::from(addr2.clone())],
        },
    );
    assert!(
        res.is_ok(),
        "Error removing from blacklist: {:?}",
        res.err()
    );

    // Verify that addresses are no longer blacklisted
    let is_addr1_blacklisted = BLACKLISTED_ADDRESSES
        .may_load(deps.as_ref().storage, addr1)
        .unwrap();
    let is_addr2_blacklisted = BLACKLISTED_ADDRESSES
        .may_load(deps.as_ref().storage, addr2)
        .unwrap();
    assert!(
        is_addr1_blacklisted.is_none(),
        "Address addr1 is still blacklisted"
    );
    assert!(
        is_addr2_blacklisted.is_none(),
        "Address addr2 is still blacklisted"
    );
}

#[test]
fn test_check_if_address_is_blacklisted() {
    let mut deps = dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let admin = deps.api.addr_make("admin");
    let addr1 = deps.api.addr_make("addr1");
    let addr2 = deps.api.addr_make("addr2");

    // Initialize config with owner
    let config = Config {
        name: String::from("Test Config"),
        description: String::from("Testing blacklist functionality"),
        owner: admin.clone(),
        denom: String::from("testdenom"),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Add an address to the blacklist
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&admin, &[]),
        ExecuteMsg::AddToBlacklist {
            addresses: vec![addr1.to_string()],
        },
    );
    assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

    // Query if the address is blacklisted
    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsAddressBlacklisted {
            address: addr1.to_string(),
        },
    );
    assert!(
        query_res.is_ok(),
        "Error querying blacklist status: {:?}",
        query_res.err()
    );

    let is_blacklisted: bool = from_json(query_res.unwrap()).unwrap();
    assert!(is_blacklisted, "Address addr1 should be blacklisted");

    // Query an address that is not blacklisted
    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsAddressBlacklisted {
            address: addr2.to_string(),
        },
    );
    assert!(
        query_res.is_ok(),
        "Error querying blacklist status: {:?}",
        query_res.err()
    );

    let is_blacklisted: bool = from_json(query_res.unwrap()).unwrap();
    assert!(!is_blacklisted, "Address addr2 should not be blacklisted");
}

#[test]
fn test_total_vp_excludes_blacklisted_addresses() {
    let mut deps = dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let admin = deps.api.addr_make("admin");
    let delegator1 = deps.api.addr_make("delegator1");
    let delegator2 = deps.api.addr_make("delegator2");

    let env = mock_env();

    let config = Config {
        name: "Test Vault".to_string(),
        description: "Testing vault functionality".to_string(),
        owner: admin.clone(),
        denom: "token".to_string(),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Define consensus and operator addresses
    let cons_addr1 = Addr::unchecked("neutronvalcons1xyz");
    let oper_addr1 = Addr::unchecked("neutronvaloper1xyz");
    let cons_addr2 = Addr::unchecked("neutronvalcons2xyz");
    let oper_addr2 = Addr::unchecked("neutronvaloper2xyz");

    // Add validators using consensus address as the key
    let validator1 = Validator {
        cons_address: cons_addr1.clone(),
        oper_address: oper_addr1.clone(),
        bonded: true,
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
        active: true,
    };
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &cons_addr1,
            &validator1,
            env.block.height,
        )
        .unwrap();

    let validator2 = Validator {
        cons_address: cons_addr2.clone(),
        oper_address: oper_addr2.clone(),
        bonded: true,
        total_tokens: Uint128::new(500),
        total_shares: Uint128::new(500),
        active: true,
    };
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &cons_addr2,
            &validator2,
            env.block.height,
        )
        .unwrap();

    let delegation1 = Delegation {
        delegator_address: delegator1.clone(),
        validator_address: oper_addr1.clone(), // Uses operator address
        shares: Uint128::new(500),
    };
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator1, &oper_addr1),
            &delegation1,
            env.block.height,
        )
        .unwrap();

    let delegation2 = Delegation {
        delegator_address: delegator2.clone(),
        validator_address: oper_addr2.clone(), // Uses operator address
        shares: Uint128::new(500),
    };
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator2, &oper_addr2),
            &delegation2,
            env.block.height,
        )
        .unwrap();

    // Query total voting power **before** blacklisting anything
    let initial_query_res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::TotalPowerAtHeight {
            height: Some(env.block.height + 1),
        },
    );
    assert!(
        initial_query_res.is_ok(),
        "Error querying total power before blacklisting: {:?}",
        initial_query_res.err()
    );

    let initial_total_power: Uint128 = from_json(initial_query_res.unwrap()).unwrap();

    // Expected power: sum of both validator tokens (1000 + 500 = 1500)
    assert_eq!(
        initial_total_power,
        Uint128::new(1500),
        "Initial total power should be sum of both validators' tokens"
    );

    // Blacklist address "addr2"
    let res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&admin, &[]),
        ExecuteMsg::AddToBlacklist {
            addresses: vec![delegator2.to_string()],
        },
    );
    assert!(res.is_ok(), "Error adding to blacklist: {:?}", res.err());

    // Ensure validator1 still exists and has tokens
    let validator1_state = VALIDATORS.load(deps.as_ref().storage, &cons_addr1).unwrap();
    assert_eq!(
        validator1_state.total_tokens,
        Uint128::new(1000),
        "Validator1's tokens are incorrect"
    );

    // Ensure validator2 still exists
    let validator2_state = VALIDATORS.load(deps.as_ref().storage, &cons_addr2).unwrap();
    assert_eq!(
        validator2_state.total_tokens,
        Uint128::new(500),
        "Validator2's tokens are incorrect"
    );

    // Ensure delegation1 is still present
    let delegation1_state = DELEGATIONS
        .load(deps.as_ref().storage, (&delegator1, &oper_addr1))
        .unwrap();
    assert_eq!(
        delegation1_state.shares,
        Uint128::new(500),
        "Delegation1 shares incorrect"
    );

    // Ensure delegation2 is blacklisted correctly
    let is_blacklisted = BLACKLISTED_ADDRESSES
        .load(deps.as_ref().storage, delegator2.clone())
        .unwrap_or(false);
    assert!(is_blacklisted, "Delegator2 should be blacklisted");

    // Query total voting power **after** blacklisting
    let query_res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::TotalPowerAtHeight {
            height: Some(env.block.height + 1),
        },
    );
    assert!(
        query_res.is_ok(),
        "Error querying total power after blacklisting: {:?}",
        query_res.err()
    );

    let total_power: Uint128 = from_json(query_res.unwrap()).unwrap();

    // Only validator1's power should count (1000), validator2's delegation is blacklisted
    assert_eq!(
        total_power,
        Uint128::new(1000),
        "Total power should exclude blacklisted address"
    );
}

#[test]
fn test_after_validator_bonded_with_mock_query() {
    let mut deps = dependencies(); // Using the mock_dependencies function
    let env = mock_env(); // Creating a mock environment

    // Define operator (valoper) and consensus (valcons) addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let cons_addr = Addr::unchecked("neutronvalcons1xyz");

    // Store an initial validator state with `bonded = false`
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &oper_addr,
            &Validator {
                cons_address: cons_addr.clone(),
                oper_address: oper_addr.clone(),
                bonded: false,
                total_tokens: Uint128::zero(),
                total_shares: Uint128::zero(),
                active: true,
            },
            env.block.height,
        )
        .unwrap();

    // Mock the validator query response
    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        status: 3, // Bonded status
        tokens: "1000".to_string(),
        jailed: false,
        delegator_shares: "1000".to_string(),
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };
    deps.querier.with_validators(vec![proto_validator]);

    // Call `after_validator_bonded`
    let res = after_validator_bonded(
        deps.as_mut(),
        env.clone(),
        cons_addr.to_string(),
        oper_addr.to_string(),
    );
    assert!(res.is_ok(), "Error: {:?}", res.err());

    // Load updated validator state
    let updated_validator = VALIDATORS.load(deps.as_ref().storage, &oper_addr).unwrap(); //  Now using operator address as the key
    assert!(updated_validator.bonded, "Validator should be bonded");
    assert!(updated_validator.active, "Validator should remain active");
    assert_eq!(updated_validator.total_tokens, Uint128::new(1000));
    assert_eq!(updated_validator.total_shares, Uint128::new(1000));

    // Ensure response attributes match expected values
    let response = res.unwrap();
    assert_eq!(
        response.attributes,
        vec![
            ("action", "validator_bonded"),
            ("valcons_address", &*cons_addr.to_string()), // Match contract's attribute key
            ("valoper_address", &*oper_addr.to_string()), // Match contract's attribute key
            ("total_tokens", "1000"),
            ("total_shares", "1000"),
        ]
    );
}

#[test]
fn test_before_validator_slashed_no_delegations() {
    let mut deps = mock_dependencies();

    // Define consensus and operator addresses
    let cons_addr = Addr::unchecked("neutronvalcons1xyz");
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");

    // Add a validator with no delegations, stored by consensus address
    let validator = Validator {
        cons_address: cons_addr.clone(),
        oper_address: oper_addr.clone(),
        bonded: true,
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
        active: true,
    };
    VALIDATORS
        .save(deps.as_mut().storage, &cons_addr, &validator, 0)
        .unwrap();

    let slashing_fraction = Decimal256::percent(10); // 10% slashing

    // Call `before_validator_slashed` with the validator's addresses
    let res = before_validator_slashed(
        deps.as_mut(),
        mock_env(),
        oper_addr.to_string(),
        slashing_fraction,
    );

    // Since there are no delegations, the function should return an error
    assert!(res.is_err(), "Expected error but got: {:?}", res.ok());
}

#[test]
fn test_before_validator_slashed() {
    let mut deps = dependencies();
    let env = mock_env();

    // Define operator and consensus addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let cons_addr = Addr::unchecked("neutronvalcons1xyz");

    // Store validator using `valoper_address` as the key
    let validator = Validator {
        cons_address: cons_addr.clone(), // Still stored inside struct
        oper_address: oper_addr.clone(),
        bonded: true,
        total_tokens: Uint128::new(500),
        total_shares: Uint128::new(500), // Shares remain constant after slashing
        active: true,
    };
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &oper_addr, //  Using `valoper_address` as the key
            &validator,
            env.block.height,
        )
        .unwrap();

    // Store delegation using `valoper_address`
    let delegator_addr = Addr::unchecked("delegator1");
    let delegation = Delegation {
        delegator_address: delegator_addr.clone(),
        validator_address: oper_addr.clone(), // Uses `valoper_address`
        shares: Uint128::new(500),            // Shares do not change after slashing
    };
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator_addr, &oper_addr), // Using `valoper_address`
            &delegation,
            env.block.height,
        )
        .unwrap();

    let slashing_fraction = Decimal256::percent(10); // 10% slashing

    // Mock validator query before calling `before_validator_slashed`
    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        status: 3,                 // Bonded status
        tokens: "450".to_string(), // 10% slashed, from 500 → 450
        jailed: false,
        delegator_shares: "500".to_string(), // Shares remain 500
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };

    deps.querier.with_validators(vec![proto_validator.clone()]);

    // Mock delegation query result
    deps.querier.with_delegations(HashMap::from([(
        (delegator_addr.to_string(), oper_addr.to_string()),
        Uint128::new(500), // Ensure delegation data is available
    )]));

    // Call `before_validator_slashed`
    let res = before_validator_slashed(
        deps.as_mut(),
        env.clone(),
        oper_addr.to_string(),
        slashing_fraction,
    );
    assert!(res.is_ok(), "Error: {:?}", res.err());

    // Validate the updated validator state (using `valoper_address` as the key)
    let updated_validator = VALIDATORS.load(deps.as_ref().storage, &oper_addr).unwrap();
    assert_eq!(updated_validator.total_tokens, Uint128::new(450)); // Tokens reduced
    assert_eq!(updated_validator.total_shares, Uint128::new(500)); // Shares remain the same

    // Validate the updated delegation state
    let updated_delegation = DELEGATIONS
        .load(deps.as_ref().storage, (&delegator_addr, &oper_addr))
        .unwrap();
    assert_eq!(updated_delegation.shares, Uint128::new(500)); // Shares remain unchanged

    // Validate the response attributes
    let response = res.unwrap();
    let expected_attributes = vec![
        ("action", "before_validator_slashed"),
        ("valoper_address", "neutronvaloper1xyz"),
        ("cons_address", "neutronvalcons1xyz"),
        ("total_tokens", "450"),
        ("total_shares", "500"),
        ("slashing_fraction", "0.1"),
    ];

    // Convert `response.attributes` from `Vec<Attribute>` to `Vec<(&str, &str)>`
    let actual_attributes: Vec<(&str, &str)> = response
        .attributes
        .iter()
        .map(|attr| (attr.key.as_str(), attr.value.as_str()))
        .collect();

    // Assert that both vectors match
    assert_eq!(actual_attributes, expected_attributes);
}

#[test]
fn test_before_validator_slashed_with_multiple_delegations() {
    let mut deps = dependencies();
    let env = mock_env();

    // Define operator address (primary key for validators now)
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let cons_addr = Addr::unchecked("neutronvalcons1xyz");

    // Store validator using operator address as the key
    let validator = Validator {
        cons_address: cons_addr.clone(),
        oper_address: oper_addr.clone(),
        bonded: true,
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000), // Shares remain constant
        active: true,
    };
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &oper_addr, //  Now using `oper_addr` as primary key
            &validator,
            env.block.height,
        )
        .unwrap();

    // Store multiple delegations using operator address
    let delegator1 = Addr::unchecked("delegator1");
    let delegator2 = Addr::unchecked("delegator2");

    let delegation1 = Delegation {
        delegator_address: delegator1.clone(),
        validator_address: oper_addr.clone(), // Uses operator address
        shares: Uint128::new(400),
    };
    let delegation2 = Delegation {
        delegator_address: delegator2.clone(),
        validator_address: oper_addr.clone(),
        shares: Uint128::new(600),
    };

    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator1, &oper_addr),
            &delegation1,
            env.block.height,
        )
        .unwrap();
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator2, &oper_addr),
            &delegation2,
            env.block.height,
        )
        .unwrap();

    let slashing_fraction = Decimal256::percent(10); // 10% slashing

    // Mock validator query to reflect slashed tokens
    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        status: 3,                 // Bonded status
        tokens: "900".to_string(), // 10% slashed, from 1000 → 900
        jailed: false,
        delegator_shares: "1000".to_string(), // Shares remain 1000
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };
    deps.querier.with_validators(vec![proto_validator]);

    // Mock delegation query results (no change in shares)
    deps.querier.with_delegations(HashMap::from([
        (
            (delegator1.to_string(), oper_addr.to_string()),
            Uint128::new(400),
        ),
        (
            (delegator2.to_string(), oper_addr.to_string()),
            Uint128::new(600),
        ),
    ]));

    // Call `before_validator_slashed`
    let res = before_validator_slashed(
        deps.as_mut(),
        env.clone(),
        oper_addr.to_string(),
        slashing_fraction,
    );
    assert!(res.is_ok(), "Error: {:?}", res.err());

    // Validate updated validator state
    let updated_validator = VALIDATORS.load(deps.as_ref().storage, &oper_addr).unwrap();
    assert_eq!(updated_validator.total_tokens, Uint128::new(900)); // Tokens reduced
    assert_eq!(updated_validator.total_shares, Uint128::new(1000)); // Shares remain the same

    // Validate updated delegation states
    let updated_delegation1 = DELEGATIONS
        .load(deps.as_ref().storage, (&delegator1, &oper_addr))
        .unwrap();
    assert_eq!(updated_delegation1.shares, Uint128::new(400)); // Shares remain unchanged

    let updated_delegation2 = DELEGATIONS
        .load(deps.as_ref().storage, (&delegator2, &oper_addr))
        .unwrap();
    assert_eq!(updated_delegation2.shares, Uint128::new(600)); // Shares remain unchanged

    // Ensure validator total shares match the sum of all delegations
    let total_delegation_shares = updated_delegation1.shares + updated_delegation2.shares;
    assert_eq!(
        updated_validator.total_shares, total_delegation_shares,
        "Validator total shares do not match the sum of all delegations!"
    );

    // Ensure validator tokens are correctly reduced by 10%
    assert_eq!(
        updated_validator.total_tokens,
        Uint128::new(900),
        "Validator total tokens do not match expected value after slashing!"
    );

    // Validate response attributes
    let response = res.unwrap();

    // Convert response attributes into comparable format
    let actual_attributes: Vec<(&str, &str)> = response
        .attributes
        .iter()
        .map(|attr| (attr.key.as_str(), attr.value.as_str()))
        .collect();

    let slashing_fraction_str = slashing_fraction.to_string(); // Store the string first

    let expected_attributes = vec![
        ("action", "before_validator_slashed"),
        ("valoper_address", oper_addr.as_str()),
        ("cons_address", cons_addr.as_str()),
        ("total_tokens", "900"),
        ("total_shares", "1000"),
        ("slashing_fraction", slashing_fraction_str.as_str()), // Use the stored string
    ];

    assert_eq!(actual_attributes, expected_attributes);

    assert_eq!(actual_attributes, expected_attributes);
}

#[test]
fn test_after_validator_created_with_mock_query() {
    let mut deps = dependencies();

    let env = mock_env();

    // Define operator (valoper) and consensus (valcons) addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");

    // Mock the validator query response
    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        jailed: false,
        status: 2, // Unbonded status
        tokens: "1000".to_string(),
        delegator_shares: "1000".to_string(),
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };

    deps.querier.with_validators(vec![proto_validator]);

    // Call `after_validator_created`
    let res = after_validator_created(deps.as_mut(), env.clone(), oper_addr.to_string());
    assert!(res.is_ok(), "Error: {:?}", res.err());

    let validator = VALIDATORS.load(deps.as_ref().storage, &oper_addr).unwrap();
    assert_eq!(
        validator.oper_address, oper_addr,
        "Operator address mismatch"
    );
    assert!(
        !validator.bonded,
        "Validator should not be bonded initially"
    );
    assert_eq!(
        validator.total_tokens,
        Uint128::new(1000),
        "Total tokens do not match the mocked data"
    );
    assert_eq!(
        validator.total_shares,
        Uint128::new(1000),
        "Total shares do not match the mocked data"
    );
    assert!(validator.active, "Validator should be active");
}

#[test]
fn test_before_delegation_removed() {
    let mut deps = dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let delegator3 = deps.api.addr_make("delegator3");

    let env = mock_env();

    // Define consensus and operator addresses
    let cons_addr = Addr::unchecked("neutronvalcons5xyz");
    let oper_addr = Addr::unchecked("neutronvaloper5xyz");

    // Store validator using `valoper`
    let validator = Validator {
        cons_address: cons_addr.clone(),
        oper_address: oper_addr.clone(),
        bonded: true,
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
        active: true,
    };
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &oper_addr,
            &validator,
            env.block.height,
        )
        .unwrap();

    // Store delegation using `valoper`
    let delegator_addr = delegator3;
    let delegation = Delegation {
        delegator_address: delegator_addr.clone(),
        validator_address: oper_addr.clone(), //  Uses valoper
        shares: Uint128::new(500),
    };
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator_addr, &oper_addr), //  Uses valoper
            &delegation,
            env.block.height,
        )
        .unwrap();

    // Mock validator query response (reflecting updated token state after removal)
    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        status: 3,                 // Bonded status
        tokens: "500".to_string(), // 50% slashed (from 1000 to 500)
        jailed: false,
        delegator_shares: "500".to_string(), // Updated shares after removal
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };

    deps.querier.with_validators(vec![proto_validator]);

    // Mock delegation query response
    deps.querier.with_delegations(HashMap::from([(
        (delegator_addr.to_string(), oper_addr.to_string()),
        Uint128::zero(), // Indicating delegation has been removed
    )]));

    // Call `before_delegation_removed`
    let res = before_delegation_removed(
        deps.as_mut(),
        env.clone(),
        delegator_addr.to_string(),
        oper_addr.to_string(),
    );

    //  Check if execution was successful
    assert!(res.is_ok(), "Error: {:?}", res.err());

    //  Validate updated validator state
    let updated_validator = VALIDATORS.load(deps.as_ref().storage, &oper_addr).unwrap();
    assert_eq!(
        updated_validator.total_tokens,
        Uint128::new(500),
        "Validator total tokens should be reduced to 500"
    );
    assert_eq!(
        updated_validator.total_shares,
        Uint128::new(500),
        "Validator total shares should be reduced to 500"
    );
}

#[test]
fn test_create_delegation_and_query_voting_power_direct_write() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();

    // Define Consensus (`valcons`) and Operator (`valoper`) addresses
    let cons_addr = Addr::unchecked("neutronvalcons1xyz");
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let delegator_addr = Addr::unchecked("delegator1");

    // Store validator directly in state (Using consensus address as key)
    let validator = Validator {
        cons_address: cons_addr.clone(), // `valcons`
        oper_address: oper_addr.clone(), // `valoper`
        bonded: true,
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
        active: true,
    };
    VALIDATORS
        .save(deps.as_mut().storage, &cons_addr, &validator, 10) // Store by consensus address
        .unwrap();

    // Store delegation directly in state (Using operator address)
    let delegation = Delegation {
        delegator_address: delegator_addr.clone(),
        validator_address: oper_addr.clone(), // Uses `valoper`
        shares: Uint128::new(500),
    };
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator_addr, &oper_addr), // Stored using `valoper`
            &delegation,
            10,
        )
        .unwrap();

    // 🔍 Query **current** voting power
    let query_response = query_voting_power_at_height(
        deps.as_ref(),
        env.clone(),
        delegator_addr.clone(),
        Some(env.block.height + 1), // Latest height
    );
    assert!(query_response.is_ok(), "Failed to query voting power");

    let query_res = query_response.unwrap();
    assert_eq!(
        query_res, delegation.shares,
        "Delegator voting power mismatch"
    );
    // assert_eq!(query_res.height, env.block.height, "Unexpected query height");

    // Query **total voting power** at current height
    let total_power_res = query_total_power_at_height(deps.as_ref(), env.clone(), None);
    assert!(total_power_res.is_ok(), "Failed to query total power");

    let total_power_response = total_power_res.unwrap();
    assert_eq!(
        total_power_response, validator.total_tokens,
        "Total voting power mismatch"
    );
    // assert_eq!(total_power_response.height, env.block.height, "Unexpected query height");

    // Simulate passage of time (historical queries)
    let historical_height = 11;
    env.block.height = historical_height;

    // Query **historical** voting power
    let historical_vp_res = query_voting_power_at_height(
        deps.as_ref(),
        env.clone(),
        delegator_addr.clone(),
        Some(historical_height),
    );
    assert!(
        historical_vp_res.is_ok(),
        "Failed to query historical voting power"
    );

    let historical_vp = historical_vp_res.unwrap();
    assert_eq!(
        historical_vp, delegation.shares,
        "Historical voting power mismatch"
    );
    // assert_eq!(historical_vp.height, historical_height, "Unexpected historical height");

    // 🔍 Query **historical** total power
    let historical_total_power_res =
        query_total_power_at_height(deps.as_ref(), env.clone(), Some(historical_height));
    assert!(
        historical_total_power_res.is_ok(),
        "Failed to query historical total power"
    );

    let historical_total_power = historical_total_power_res.unwrap();
    assert_eq!(
        historical_total_power, validator.total_tokens,
        "Historical total power mismatch"
    );
    // assert_eq!(historical_total_power.height, historical_height, "Unexpected historical height");
}

#[test]
fn test_after_delegation_modified() {
    let mut deps = dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let delegator1 = deps.api.addr_make("delegator1");

    let mut env = mock_env();

    // Define operator (valoper) and consensus (valcons) addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let cons_addr = Addr::unchecked("neutronvalcons1xyz");
    let delegator_addr = delegator1;

    // Store validator in the state using valoper as the primary key
    let validator = Validator {
        cons_address: cons_addr.clone(),
        oper_address: oper_addr.clone(),
        bonded: true,
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
        active: true,
    };
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &oper_addr,
            &validator,
            env.block.height,
        )
        .unwrap();

    // Store initial delegation to ensure it exists before calling `after_delegation_modified`
    let initial_delegation = Delegation {
        delegator_address: delegator_addr.clone(),
        validator_address: oper_addr.clone(),
        shares: Uint128::new(100), // Initial shares before modification
    };
    DELEGATIONS
        .save(
            deps.as_mut().storage,
            (&delegator_addr, &oper_addr),
            &initial_delegation,
            env.block.height,
        )
        .unwrap();

    // Mock validator query response
    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        status: 3,                  // Bonded status
        tokens: "1200".to_string(), // Updated tokens after delegation
        jailed: false,
        delegator_shares: "1200".to_string(), // Updated shares
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };
    deps.querier.with_validators(vec![proto_validator]);

    // **Mock delegation query response with updated shares**
    deps.querier.with_delegations(HashMap::from([(
        (delegator_addr.to_string(), oper_addr.to_string()),
        Uint128::new(200), // New delegation amount
    )]));

    // Call `after_delegation_modified`
    let res = after_delegation_modified(
        deps.as_mut(),
        env.clone(),
        delegator_addr.to_string(),
        oper_addr.to_string(),
    );

    assert!(res.is_ok(), "Error: {:?}", res.err());

    // Validate updated validator state
    let updated_validator = VALIDATORS.load(deps.as_ref().storage, &oper_addr).unwrap();
    assert_eq!(updated_validator.total_tokens, Uint128::new(1200)); // Tokens updated
    assert_eq!(updated_validator.total_shares, Uint128::new(1200)); // Shares updated

    // Validate updated delegation state
    let updated_delegation = DELEGATIONS
        .load(deps.as_ref().storage, (&delegator_addr, &oper_addr))
        .unwrap();
    assert_eq!(updated_delegation.shares, Uint128::new(200)); // New delegation shares

    // Validate response attributes
    let response = res.unwrap();
    assert_eq!(
        response.attributes,
        vec![
            ("action", "after_delegation_modified"),
            ("delegator", delegator_addr.to_string().as_str()),
            ("cons_address", cons_addr.to_string().as_str()),
            ("valoper_address", oper_addr.to_string().as_str()),
            ("total_shares", "1200"),
            ("total_tokens", "1200"),
            ("delegation_shares", "200"),
        ]
    );

    env.block.height += 5;

    let total_voting_power = query_total_power_at_height(deps.as_ref(), env.clone(), None).unwrap();
    assert_eq!(total_voting_power, Uint128::new(1200));
    let voting_power =
        query_voting_power_at_height(deps.as_ref(), env.clone(), delegator_addr.clone(), None)
            .unwrap();
    assert_eq!(voting_power, Uint128::new(200));

    env.block.height += 5;

    //-----------------------------------------------------------------
    // **Mock delegation query response with updated shares**
    deps.querier.with_delegations(HashMap::from([(
        (delegator_addr.to_string(), oper_addr.to_string()),
        Uint128::new(100), // New delegation amount
    )]));

    // Call `after_delegation_modified`
    let res = after_delegation_modified(
        deps.as_mut(),
        env.clone(),
        delegator_addr.to_string(),
        oper_addr.to_string(),
    );

    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        status: 3,                  // Bonded status
        tokens: "1100".to_string(), // Updated tokens after delegation
        jailed: false,
        delegator_shares: "1100".to_string(), // Updated shares
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };
    deps.querier.with_validators(vec![proto_validator]);

    assert!(res.is_ok(), "Error: {:?}", res.err());

    // Validate updated delegation state
    let updated_delegation = DELEGATIONS
        .load(deps.as_ref().storage, (&delegator_addr, &oper_addr))
        .unwrap();
    assert_eq!(updated_delegation.shares, Uint128::new(100)); // New delegation shares

    env.block.height += 5;

    let total_voting_power = query_total_power_at_height(deps.as_ref(), env.clone(), None).unwrap();
    assert_eq!(total_voting_power, Uint128::new(1100));
    let voting_power =
        query_voting_power_at_height(deps.as_ref(), env.clone(), delegator_addr, None).unwrap();
    assert_eq!(voting_power, Uint128::new(100));
}
