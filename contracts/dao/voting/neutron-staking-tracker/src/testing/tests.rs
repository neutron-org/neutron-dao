#[cfg(test)]
use crate::contract::{
    after_delegation_modified, after_validator_begin_unbonding, after_validator_bonded,
    before_validator_slashed, execute, instantiate, query_total_power_at_height,
    query_voting_power_at_height,
};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{
    Config, Delegation, Validator, BONDED_VALIDATORS, CONFIG, DELEGATIONS, VALIDATORS,
};
use crate::testing::mock_querier::mock_dependencies as dependencies;
use cosmwasm_std::testing::message_info;
use cosmwasm_std::{
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
        staking_proxy_info_contract_address: None,
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
    assert_eq!(config.owner, valid_neutron_address);
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
        staking_proxy_info_contract_address: None,
    };
    let info = message_info(creator, &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let new_owner = deps.api.addr_make("new_owner");
    let new_staking_proxy_info_contract_address = deps
        .api
        .addr_make("new_staking_proxy_info_contract_address");
    // Update config with correct owner
    let update_msg = ExecuteMsg::UpdateConfig {
        owner: Some(new_owner.to_string()),
        name: Some("Updated DAO".to_string()),
        description: Some("Updated description".to_string()),
        staking_proxy_info_contract_address: Some(
            new_staking_proxy_info_contract_address.to_string(),
        ),
    };
    let info = message_info(owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_msg);
    assert!(res.is_ok());

    // Validate updated config
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.name, "Updated DAO");
    assert_eq!(config.description, "Updated description");
    assert_eq!(config.owner, new_owner);
    assert_eq!(
        config.staking_proxy_info_contract_address,
        Some(new_staking_proxy_info_contract_address)
    );
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("neutron");

    let owner = &deps.api.addr_make("owner");
    let new_staking_proxy_info_contract_address = deps
        .api
        .addr_make("new_staking_proxy_info_contract_address");
    let new_owner = &deps.api.addr_make("new_owner");
    let creator = &deps.api.addr_make("creator");
    let unauthorized = &deps.api.addr_make("unauthorized");

    // Initialize the contract
    let msg = InstantiateMsg {
        name: "Test DAO".to_string(),
        description: "A test DAO contract".to_string(),
        owner: owner.to_string(),
        staking_proxy_info_contract_address: None,
    };
    let info = message_info(creator, &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Try to update config with wrong owner
    let update_msg = ExecuteMsg::UpdateConfig {
        owner: Some(new_owner.to_string()),
        name: Some("Updated DAO".to_string()),
        description: Some("Updated description".to_string()),
        staking_proxy_info_contract_address: Some(
            new_staking_proxy_info_contract_address.to_string(),
        ),
    };
    let info = message_info(unauthorized, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_msg);
    assert!(res.is_err());

    // Ensure config is unchanged
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.name, "Test DAO");
    assert_eq!(config.description, "A test DAO contract");
    assert_eq!(config.owner, owner);
    assert_eq!(config.staking_proxy_info_contract_address, None);
}

#[test]
fn test_after_validator_bonded_with_mock_query() {
    let mut deps = dependencies(); // Using the mock_dependencies function
    let env = mock_env(); // Creating a mock environment

    // store CONFIG
    let config = Config {
        name: "Test Vault".to_string(),
        description: "Testing vault functionality".to_string(),
        owner: deps.api.addr_make("admin"),
        staking_proxy_info_contract_address: Some(
            deps.api.addr_make("staking_proxy_info_contract_address"),
        ),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Define operator (valoper) and consensus (valcons) addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let cons_addr = Addr::unchecked("neutronvalcons1xyz");

    BONDED_VALIDATORS
        .save(deps.as_mut().storage, &Vec::new(), env.block.height)
        .unwrap();
    // Store an initial validator state with `bonded = false`
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &oper_addr,
            &Validator {
                oper_address: oper_addr.clone(),
                total_tokens: Uint128::zero(),
                total_shares: Uint128::zero(),
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
    assert!(BONDED_VALIDATORS
        .load(deps.as_ref().storage)
        .unwrap()
        .contains(&updated_validator.oper_address.to_string()));
    assert_eq!(updated_validator.total_tokens, Uint128::new(1000));
    assert_eq!(updated_validator.total_shares, Uint128::new(1000));

    // Ensure response attributes match expected values
    let response = res.unwrap();
    assert_eq!(
        response.attributes,
        vec![
            ("action", "validator_bonded"),
            ("valoper_address", &*oper_addr.to_string()), // Match contract's attribute key
            ("total_tokens", "1000"),
            ("total_shares", "1000"),
        ]
    );
}

#[test]
fn test_before_validator_slashed_with_self_bonded_only() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    // store CONFIG
    let config = Config {
        name: "Test Vault".to_string(),
        description: "Testing vault functionality".to_string(),
        owner: deps.api.addr_make("admin"),
        staking_proxy_info_contract_address: Some(
            deps.api.addr_make("staking_proxy_info_contract_address"),
        ),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Define consensus and operator addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");

    // Validator has self-bonded tokens but no external delegations
    BONDED_VALIDATORS
        .save(deps.as_mut().storage, &vec![oper_addr.to_string()], 0)
        .unwrap();
    let validator = Validator {
        oper_address: oper_addr.clone(),
        total_tokens: Uint128::new(1000), // Self-bonded tokens
        total_shares: Uint128::new(1000), // No external delegators
    };

    // Store validator state by operator address
    VALIDATORS
        .save(deps.as_mut().storage, &oper_addr, &validator, 0)
        .unwrap();

    let slashing_fraction = Decimal256::percent(10); // 10% slashing

    // Call `before_validator_slashed` with the validator’s address
    let res = before_validator_slashed(
        deps.as_mut(),
        env.clone(),
        oper_addr.to_string(),
        slashing_fraction,
    );

    assert!(
        res.is_ok(),
        "Expected successful execution but got error: {:?}",
        res.err()
    );

    // Validate the updated validator state
    let updated_validator = VALIDATORS.load(deps.as_ref().storage, &oper_addr).unwrap();
    assert_eq!(updated_validator.total_tokens, Uint128::new(900)); // 10% slashed
    assert_eq!(updated_validator.total_shares, Uint128::new(1000)); // Shares remain the same

    // Validate response attributes
    let response = res.unwrap();
    let expected_attributes = vec![
        ("action", "before_validator_slashed"),
        ("valoper_address", "neutronvaloper1xyz"),
        ("total_tokens", "900"),      // 10% slashed from 1000 → 900
        ("total_shares", "1000"),     // Shares remain unchanged
        ("slashing_fraction", "0.1"), // 10% slashing
    ];

    // Convert response attributes for assertion
    let actual_attributes: Vec<(&str, &str)> = response
        .attributes
        .iter()
        .map(|attr| (attr.key.as_str(), attr.value.as_str()))
        .collect();

    assert_eq!(actual_attributes, expected_attributes);
}

#[test]
fn test_before_validator_slashed() {
    let mut deps = dependencies();
    let env = mock_env();

    // store CONFIG
    let config = Config {
        name: "Test Vault".to_string(),
        description: "Testing vault functionality".to_string(),
        owner: deps.api.addr_make("admin"),
        staking_proxy_info_contract_address: Some(
            deps.api.addr_make("staking_proxy_info_contract_address"),
        ),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Define operator and consensus addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");

    // Store validator using `valoper_address` as the key
    BONDED_VALIDATORS
        .save(
            deps.as_mut().storage,
            &vec![oper_addr.to_string()],
            env.block.height,
        )
        .unwrap();
    let validator = Validator {
        oper_address: oper_addr.clone(),
        total_tokens: Uint128::new(500),
        total_shares: Uint128::new(500), // Shares remain constant after slashing
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
fn test_before_validator_slashed_voting_power_drops() {
    let mut deps = dependencies();
    let env = mock_env();

    // store CONFIG
    let config = Config {
        name: "Test Vault".to_string(),
        description: "Testing vault functionality".to_string(),
        owner: deps.api.addr_make("admin"),
        staking_proxy_info_contract_address: Some(
            deps.api.addr_make("staking_proxy_info_contract_address"),
        ),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Define operator address (primary key for validators now)
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");

    // Initial validator state
    BONDED_VALIDATORS
        .save(
            deps.as_mut().storage,
            &vec![oper_addr.to_string()],
            env.block.height,
        )
        .unwrap();
    let validator = Validator {
        oper_address: oper_addr.clone(),
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
    };

    // Store validator using operator address as the key
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &oper_addr,
            &validator,
            env.block.height,
        )
        .unwrap();

    // Store multiple delegations using operator address
    let delegator1 = Addr::unchecked("delegator1");
    let delegator2 = Addr::unchecked("delegator2");

    let delegation1 = Delegation {
        delegator_address: delegator1.clone(),
        validator_address: oper_addr.clone(),
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

    // Calculate voting power BEFORE slashing
    let voting_power_before_1 =
        delegation1.shares * validator.total_tokens / validator.total_shares;
    let voting_power_before_2 =
        delegation2.shares * validator.total_tokens / validator.total_shares;

    // Mock validator query to reflect slashed tokens
    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        status: 3,                 // Bonded status
        tokens: "900".to_string(), // 10% slashed, from 1000 → 900
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

    // Call before_validator_slashed
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

    // Calculate voting power AFTER slashing
    let voting_power_after_1 = updated_delegation1.shares * updated_validator.total_tokens
        / updated_validator.total_shares;
    let voting_power_after_2 = updated_delegation2.shares * updated_validator.total_tokens
        / updated_validator.total_shares;

    // Ensure delegators' voting power decreased
    assert!(
        voting_power_after_1 < voting_power_before_1,
        "Delegator 1's voting power did not decrease!"
    );
    assert!(
        voting_power_after_2 < voting_power_before_2,
        "Delegator 2's voting power did not decrease!"
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
        ("total_tokens", "900"),
        ("total_shares", "1000"),
        ("slashing_fraction", slashing_fraction_str.as_str()), // Use the stored string
    ];

    assert_eq!(actual_attributes, expected_attributes);
}

#[test]
fn test_create_delegation_and_query_voting_power_direct_write() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();

    // Define Consensus (`valcons`) and Operator (`valoper`) addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let delegator_addr = Addr::unchecked("delegator1");

    // Store validator directly in state (Using consensus address as key)
    BONDED_VALIDATORS
        .save(deps.as_mut().storage, &vec![oper_addr.to_string()], 10)
        .unwrap();
    let validator = Validator {
        oper_address: oper_addr.clone(), // `valoper`
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
    };
    VALIDATORS
        .save(deps.as_mut().storage, &oper_addr, &validator, 10) // Store by consensus address
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

    // store CONFIG
    let config = Config {
        name: "Test Vault".to_string(),
        description: "Testing vault functionality".to_string(),
        owner: deps.api.addr_make("admin"),
        staking_proxy_info_contract_address: Some(
            deps.api.addr_make("staking_proxy_info_contract_address"),
        ),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Define operator (valoper) and consensus (valcons) addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let delegator_addr = delegator1;

    // Store validator in the state using valoper as the primary key
    BONDED_VALIDATORS
        .save(
            deps.as_mut().storage,
            &vec![oper_addr.to_string()],
            env.block.height,
        )
        .unwrap();
    let validator = Validator {
        oper_address: oper_addr.clone(),
        total_tokens: Uint128::new(1000),
        total_shares: Uint128::new(1000),
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
            ("valoper_address", oper_addr.to_string().as_str()),
            ("delegation_shares", "200"),
            ("total_shares", "1200"),
            ("total_tokens", "1200"),
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

#[test]
fn test_after_delegation_modified_large_scaled_shares() {
    let mut deps = dependencies();
    deps.api = deps.api.with_prefix("neutron");

    // store CONFIG
    let config = Config {
        name: "Test Vault".to_string(),
        description: "Testing vault functionality".to_string(),
        owner: deps.api.addr_make("admin"),
        staking_proxy_info_contract_address: Some(
            deps.api.addr_make("staking_proxy_info_contract_address"),
        ),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    let delegator1 = deps.api.addr_make("delegator1");

    let mut env = mock_env();

    // Define operator (valoper) and consensus (valcons) addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xdlvhs2l2wq0cc3eskyxphstns3348el5l4qan");
    let delegator_addr = delegator1;

    // Store validator in the state using valoper as the primary key
    BONDED_VALIDATORS
        .save(
            deps.as_mut().storage,
            &vec![oper_addr.to_string()],
            env.block.height,
        )
        .unwrap();
    let validator = Validator {
        oper_address: oper_addr.clone(),
        total_tokens: Uint128::new(166666667666), // Tokens remain as original values
        total_shares: Uint128::new(166666667666000000000000000000), // Shares scaled up
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
        shares: Uint128::new(166666667666000000000000000000), // Initial shares before modification
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
        status: 3,                          // Bonded status
        tokens: "166666667667".to_string(), // Tokens remain unchanged
        jailed: false,
        delegator_shares: "166666667667000000000000000000".to_string(), // Shares scaled up
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
        Uint128::new(166666667667000000000000000000), // Updated delegation amount
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
    assert_eq!(updated_validator.total_tokens, Uint128::new(166666667667)); // Tokens remain unchanged
    assert_eq!(
        updated_validator.total_shares,
        Uint128::new(166666667667000000000000000000)
    ); // Shares updated

    // Validate updated delegation state
    let updated_delegation = DELEGATIONS
        .load(deps.as_ref().storage, (&delegator_addr, &oper_addr))
        .unwrap();
    assert_eq!(
        updated_delegation.shares,
        Uint128::new(166666667667000000000000000000)
    ); // New delegation shares

    // Validate response attributes
    let response = res.unwrap();
    assert_eq!(
        response.attributes,
        vec![
            ("action", "after_delegation_modified"),
            ("delegator", delegator_addr.to_string().as_str()),
            ("valoper_address", oper_addr.to_string().as_str()),
            ("delegation_shares", "166666667667000000000000000000"),
            ("total_shares", "166666667667000000000000000000"),
            ("total_tokens", "166666667667"),
        ]
    );

    env.block.height += 5;

    let total_voting_power = query_total_power_at_height(deps.as_ref(), env.clone(), None).unwrap();
    assert_eq!(total_voting_power, Uint128::new(166666667667));
    let voting_power =
        query_voting_power_at_height(deps.as_ref(), env.clone(), delegator_addr.clone(), None)
            .unwrap();
    assert_eq!(voting_power, Uint128::new(166666667667));

    env.block.height += 5;

    //-----------------------------------------------------------------
    // **Mock delegation query response with updated shares**
    deps.querier.with_delegations(HashMap::from([(
        (delegator_addr.to_string(), oper_addr.to_string()),
        Uint128::new(166666667666000000000000000000), // New delegation amount
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
        status: 3,                          // Bonded status
        tokens: "166666667666".to_string(), // Tokens remain unchanged
        jailed: false,
        delegator_shares: "166666667666000000000000000000".to_string(), // Shares scaled up
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
    assert_eq!(
        updated_delegation.shares,
        Uint128::new(166666667666000000000000000000)
    ); // New delegation shares

    env.block.height += 5;

    let total_voting_power = query_total_power_at_height(deps.as_ref(), env.clone(), None).unwrap();
    assert_eq!(total_voting_power, Uint128::new(166666667666));
    let voting_power =
        query_voting_power_at_height(deps.as_ref(), env.clone(), delegator_addr, None).unwrap();
    assert_eq!(voting_power, Uint128::new(166666667666));
}

#[test]
fn test_after_validator_begin_unbonding() {
    let mut deps = dependencies(); // Initialize dependencies
    let mut env = mock_env(); // Mock environment

    let admin = deps.api.addr_make("admin");
    // Define operator (valoper) and consensus (valcons) addresses
    let oper_addr = Addr::unchecked("neutronvaloper1xyz");
    let cons_addr = Addr::unchecked("neutronvalcons1xyz");

    let config = Config {
        name: String::from("Test Config"),
        description: String::from("Testing validator unbonding handler"),
        owner: admin.clone(),
        staking_proxy_info_contract_address: None,
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Store an initial validator state with `bonded = true`
    BONDED_VALIDATORS
        .save(
            deps.as_mut().storage,
            &vec![oper_addr.to_string()],
            env.block.height,
        )
        .unwrap();
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &oper_addr,
            &Validator {
                oper_address: oper_addr.clone(),
                total_tokens: Uint128::new(1000),
                total_shares: Uint128::new(1000),
            },
            env.block.height,
        )
        .unwrap();

    // Pass height so we don't unbond on the saving height
    env.block.height += 5;

    let unbonding_height = env.block.height;

    // Mock the validator query response (now in unbonding state)
    let proto_validator = CosmosValidator {
        operator_address: oper_addr.to_string(),
        consensus_pubkey: None,
        status: 2,                  // Unbonding status
        tokens: "1000".to_string(), // Total tokens remain the same
        jailed: false,
        delegator_shares: "1000".to_string(),
        description: None,
        unbonding_height: (env.block.height + 1) as i64,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
        unbonding_on_hold_ref_count: 0,
        unbonding_ids: vec![],
    };
    deps.querier.with_validators(vec![proto_validator]);

    // Call `after_validator_begin_unbonding`
    let res = after_validator_begin_unbonding(
        deps.as_mut(),
        env.clone(),
        cons_addr.to_string(),
        oper_addr.to_string(),
    );
    assert!(res.is_ok(), "Error: {:?}", res.err());

    // Load updated validator state
    let updated_validator = VALIDATORS.load(deps.as_ref().storage, &oper_addr).unwrap();
    assert!(
        !BONDED_VALIDATORS
            .load(deps.as_ref().storage)
            .unwrap()
            .contains(&updated_validator.oper_address.to_string()),
        "Validator should not be bonded after unbonding begins"
    );
    // Previous validator state should still be bonded
    assert!(
        BONDED_VALIDATORS
            .may_load_at_height(deps.as_ref().storage, env.block.height - 1)
            .unwrap()
            .unwrap()
            .contains(&updated_validator.oper_address.to_string()),
        "Validator should bonded before unbonding begins"
    );
    assert_eq!(
        updated_validator.total_tokens,
        Uint128::new(1000),
        "Total tokens should remain unchanged"
    );
    assert_eq!(
        updated_validator.total_shares,
        Uint128::new(1000),
        "Total shares should remain unchanged"
    );

    // Ensure response attributes match expected values
    let response = res.unwrap();
    assert_eq!(
        response.attributes,
        vec![
            ("action", "after_validator_begin_unbonding"),
            ("valoper_address", &*oper_addr.to_string()), // Match contract's attribute key
            ("unbonding_start_height", &*unbonding_height.to_string()),
        ]
    );
}
