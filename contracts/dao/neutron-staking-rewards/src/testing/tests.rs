use crate::contract::{execute, instantiate, query};
use crate::state::{CONFIG, PAUSED, STATE};
use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::{
    coin,
    testing::{message_info, mock_env},
    BankMsg, CosmosMsg, Response, Uint128,
};
use neutron_staking_rewards_common::error::ContractError;
use neutron_staking_rewards_common::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RewardsResponse};

// Helper to create a default instantiate message
fn default_init_msg(api: MockApi) -> InstantiateMsg {
    InstantiateMsg {
        owner: api.addr_make("owner").into(),
        annual_reward_rate_bps: 1000, // 10%
        blocks_per_year: 10_000,
        dao_address: api.addr_make("dao").into(),
        staking_info_proxy: api.addr_make("proxy").into(),
        staking_denom: "untrn".to_string(),
        security_address: api.addr_make("security_address").into(),
    }
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies();

    // Set up the contract.
    // Instantiate
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // Attempt to update the config from an unauthorized account.
    let non_owner = deps.api.addr_make("someone_else");
    let update_config_msg = ExecuteMsg::UpdateConfig {
        owner: None,
        annual_reward_rate_bps: Some(2000),
        blocks_per_year: None,
        staking_info_proxy: None,
        staking_denom: None,
        security_address: None,
    };
    let info_non_owner = message_info(&non_owner, &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info_non_owner,
        update_config_msg,
    )
    .unwrap_err();
    assert!(matches!(err, ContractError::Unauthorized {}));
}

#[test]
fn test_claim_rewards_no_pending() {
    let mut deps = mock_dependencies();

    // Set up the contract.
    // Instantiate
    let mut env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let proxy = deps.api.addr_make("proxy");
    let user = deps.api.addr_make("user1");

    // At block 100, set the stake.
    env.block.height += 100;
    deps.querier.update_stake(
        user.to_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let update_msg = ExecuteMsg::UpdateStake {
        user: user.to_string(),
    };
    let proxy_info = message_info(&proxy, &[]);
    let _ = execute(deps.as_mut(), env.clone(), proxy_info, update_msg).unwrap();

    // Immediately claim rewards in the same block.
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let user_info = message_info(&user, &[]);
    let res = execute(deps.as_mut(), env.clone(), user_info, claim_msg).unwrap();
    // Since no blocks have passed, no rewards are accrued and no BankMsg should be sent.
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_query_rewards_for_new_user() {
    let mut deps = mock_dependencies();

    // Set up the contract.
    // Instantiate
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // Query rewards for a user that has never updated their stake.
    let new_user = deps.api.addr_make("new_user");
    let query_msg = QueryMsg::Rewards {
        user: new_user.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards.pending_rewards.amount, Uint128::zero());
}

/// No change in block height: Global index should not advance.
#[test]
fn test_update_global_index_same_block() {
    let mut deps = mock_dependencies();

    // Set up the contract.
    // Instantiate
    let mut env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let proxy = deps.api.addr_make("proxy");
    let user = deps.api.addr_make("user1");

    // At block 100, set the stake.
    env.block.height += 100;
    deps.querier.update_stake(
        user.to_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let update_msg = ExecuteMsg::UpdateStake {
        user: user.to_string(),
    };
    let proxy_info = message_info(&proxy, &[]);
    let _ = execute(
        deps.as_mut(),
        env.clone(),
        proxy_info.clone(),
        update_msg.clone(),
    )
    .unwrap();

    // Without advancing the block, update the stake again (even if the stake remains unchanged).
    deps.querier.update_stake(
        user.to_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let _ = execute(deps.as_mut(), env.clone(), proxy_info, update_msg).unwrap();

    // Query rewards — none should have accrued because the block height did not advance.
    let query_msg = QueryMsg::Rewards {
        user: user.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards.pending_rewards.amount, Uint128::zero());
}

/// Tests the following scenario:
///     1.  A non-authorized address tries to update the user's stake (error)
///     2.  An authorized address tries to update stake for the DAO address (error)
#[test]
fn test_update_stake() {
    let mut deps = mock_dependencies();

    // Instantiate
    let mut env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // Ensure config is saved
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, deps.api.addr_make("owner"));
    assert_eq!(config.staking_info_proxy, deps.api.addr_make("proxy"));
    assert_eq!(config.dao_address, deps.api.addr_make("dao"));
    assert_eq!(config.annual_reward_rate_bps, 1000);
    assert_eq!(config.blocks_per_year, 10_000);
    assert_eq!(config.staking_denom, "untrn");

    // Update the stake information from an unauthorized address
    let info_not_proxy = message_info(&deps.api.addr_make("NOT_PROXY"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user0").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user0").into_string(),
        env.block.height,
        coin(1_000_000_000, "untrn"),
    );
    let res = execute(deps.as_mut(), env.clone(), info_not_proxy, msg_update_stake);
    assert_eq!(res.err().unwrap(), ContractError::Unauthorized {});

    // Update the stake information for the DAO
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("dao").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user0").into_string(),
        env.block.height,
        coin(1_000_000_000, "untrn"),
    );
    let res = execute(deps.as_mut(), env.clone(), info_proxy, msg_update_stake);
    assert_eq!(
        res.err().unwrap(),
        ContractError::DaoStakeChangeNotTracked {}
    );

    // contract must be self paused if an error happens during stake update
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user0").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user0").into_string(),
        env.block.height,
        coin(1_000_000_000, "untrn"),
    );

    env.block.height -= 1; // this allows update_stake msg to fail
                           // update_stake call must not raise an error
    execute(deps.as_mut(), env.clone(), info_proxy, msg_update_stake).unwrap();
    // but the contract must be paused after an errored call
    assert!(PAUSED.load(&deps.storage).unwrap());
}

/// Tests the following scenario:
///     (Yearly blocks: 10_000, APR: 10%)
///     ------------------------STEP 1------------------------------------------
///     1.  User stakes 1000
///     2.  100 blocks pass
///     3.  User queries the rewards (1 NTRN)
///     ------------------------STEP 2------------------------------------------
///     4.  100 blocks pass
///     5.  User queries the rewards (2 NTRN)
///     6.  User claims the rewards
///     7.  User queries the rewards (0 NTRN)
///     ------------------------STEP 3------------------------------------------
///     8.  100 blocks pass
///     9.  User unstakes 500
///     10. 100 blocks pass
///     11. User queries the rewards (1.5 NTRN)
///     12. User claims the rewards
///     ------------------------STEP 4------------------------------------------
///     13. User unstakes 500
///     13. 100 blocks pass
///     14. User queries the rewards (0 NTRN)
///     15. User queries the rewards (0 NTRN, no BankSend)
#[test]
fn test_single_user() {
    let mut deps = mock_dependencies();

    // Instantiate
    let mut env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // Ensure config is saved
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, deps.api.addr_make("owner"));
    assert_eq!(config.staking_info_proxy, deps.api.addr_make("proxy"));
    assert_eq!(config.dao_address, deps.api.addr_make("dao"));
    assert_eq!(config.annual_reward_rate_bps, 1000);
    assert_eq!(config.blocks_per_year, 10_000);
    assert_eq!(config.staking_denom, "untrn");

    // ------------------------STEP 1------------------------------------------

    // Update the stake information, acting as the proxy
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    // user1 delegates 1000 tokens
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user1").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user1").into_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info_proxy.clone(),
        msg_update_stake,
    )
    .unwrap();

    // BLOCKS >>> 100
    env.block.height += 100;

    // user1 queries pending rewards. Should be:
    // ((0.1                    / 10_000)          * 100)               * 1000         = 1 NTRN
    // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user1").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(1_000_000u128)
    );

    // ------------------------STEP 2------------------------------------------

    // BLOCKS >>> 100
    env.block.height += 100;

    // user1 queries pending rewards *before* claiming them. Should be:
    // ((0.1                    / 10_000)          * 200)               * 1000         = 2 NTRN
    // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user1").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(2_000_000u128)
    );

    // user1 claims
    let info_user1 = message_info(&deps.api.addr_make("user1"), &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), info_user1, claim_msg).unwrap();
    // Should have 1 BankMsg to send the user’s rewards
    assert_eq!(res.messages.len(), 1);

    // user1 queries rewards *after* claiming them, should be 0
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user1").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_resp.pending_rewards.amount, Uint128::new(0));

    // ------------------------STEP 3------------------------------------------

    // BLOCKS >>> 100
    env.block.height += 100;

    // Update the stake information, reducing user1's stake by 50%
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user1").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user1").into_string(),
        env.block.height,
        coin(500_000_000u128, "untrn"),
    );
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info_proxy.clone(),
        msg_update_stake,
    )
    .unwrap();

    // BLOCKS >>> 100
    env.block.height += 100;

    // user1 queries pending rewards *before* claiming them. Should be:
    // Pre-unstake:  ((0.1                    / 10_000)          * 100)               * 1000       = 1 NTRN
    // Post-unstake: ((0.1                    / 10_000)          * 100)               * 500        = 0.5 NTRN
    //               ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user1").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(1_500_000u128)
    );
    // user1 claims
    let info_user1 = message_info(&deps.api.addr_make("user1"), &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), info_user1, claim_msg).unwrap();
    // Should have 1 BankMsg to send the user’s rewards
    assert_eq!(res.messages.len(), 1);

    // ------------------------STEP 4------------------------------------------

    // Update the stake information, reducing user1's stake to 0
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user1").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user1").into_string(),
        env.block.height,
        coin(0, "untrn"),
    );
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info_proxy.clone(),
        msg_update_stake,
    )
    .unwrap();

    // BLOCKS >>> 100
    env.block.height += 100;

    // user1 queries pending rewards. Should be 0
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user1").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_resp.pending_rewards.amount, Uint128::new(0u128));
    // user1 claims
    let info_user1 = message_info(&deps.api.addr_make("user1"), &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), info_user1, claim_msg).unwrap();
    // Should have 0 BankMsg, since there is nothing to claim
    assert_eq!(res.messages.len(), 0);
}

/// Tests the following scenario:
///     (Yearly blocks: 10_000, APR: 10%)
///     ------------------------STEP 1------------------------------------------
///     1.  User2 stakes 1000
///     2.  User3 stakes 500
///     1.  100 blocks pass
///     2.  User2 queries the rewards (1 NTRN)
///     3.  User3 queries the rewards (0.5 NTRN)
///     4.  User2 claims the rewards (1 NTRN)
///     5.  User3 claims the rewards (0.5 NTRN)
///     6.  User2 queries the rewards (0 NTRN)
///     7.  User3 queries the rewards (0 NTRN)
///     ------------------------STEP 2------------------------------------------
///     1.  User2 stakes 1000
///     2.  User3 stakes 500
///     3.  100 blocks pass
///     4.  User2 queries the rewards (1 NTRN)
///     5.  User3 queries the rewards (0.5 NTRN)
///     6.  User3 unstakes everything
///     7.  100 blocks pass
///     8.  User2 queries the rewards (2 NTRN)
///     9.  User3 queries the rewards (0.5 NTRN)
#[test]
fn test_two_users() {
    let mut deps = mock_dependencies();

    // Instantiate
    let mut env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // Ensure config is saved
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, deps.api.addr_make("owner"));
    assert_eq!(config.staking_info_proxy, deps.api.addr_make("proxy"));
    assert_eq!(config.dao_address, deps.api.addr_make("dao"));
    assert_eq!(config.annual_reward_rate_bps, 1000);
    assert_eq!(config.blocks_per_year, 10_000);
    assert_eq!(config.staking_denom, "untrn");

    // ------------------------STEP 1------------------------------------------

    // Update the stake information for user2
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user2").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user2").into_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info_proxy.clone(),
        msg_update_stake,
    )
    .unwrap();

    // Update the stake information for user3, acting as the proxy
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user3").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user3").into_string(),
        env.block.height,
        coin(500_000_000u128, "untrn"),
    );
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info_proxy.clone(),
        msg_update_stake,
    )
    .unwrap();

    // BLOCKS >>> 100
    env.block.height += 100;

    // user2 queries pending rewards *before* claiming them. Should be:
    // ((0.1                    / 10_000)          * 100)               * 1000         = 1 NTRN
    // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user2").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(1_000_000u128)
    );

    // user3 queries pending rewards *before* claiming them. Should be:
    // ((0.1                    / 10_000)          * 100)               * 500         = 0.5 NTRN
    // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user3").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(500_000u128)
    );

    // user2 claims
    let info_user2 = message_info(&deps.api.addr_make("user2"), &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), info_user2, claim_msg).unwrap();
    assert_eq!(res.messages.len(), 1);

    // user3 claims
    let info_user3 = message_info(&deps.api.addr_make("user3"), &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), info_user3, claim_msg).unwrap();
    assert_eq!(res.messages.len(), 1);

    // user2 queries pending rewards *after* claiming them, should be 0
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user2").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_resp.pending_rewards.amount, Uint128::new(0u128));

    // user3 queries pending rewards *after* claiming them, should be 0
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user3").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_resp.pending_rewards.amount, Uint128::new(0u128));

    //     ------------------------STEP 2------------------------------------------
    // Update the stake information for user2
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user2").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user2").into_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info_proxy.clone(),
        msg_update_stake,
    )
    .unwrap();

    // Update the stake information for user3, acting as the proxy
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user3").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user3").into_string(),
        env.block.height,
        coin(500_000_000u128, "untrn"),
    );
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info_proxy.clone(),
        msg_update_stake,
    )
    .unwrap();

    // BLOCKS >>> 100
    env.block.height += 100;

    // user2 queries pending rewards *before* claiming them. Should be:
    // ((0.1                    / 10_000)          * 100)               * 1000         = 1 NTRN
    // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user2").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(1_000_000u128)
    );

    // user3 queries pending rewards *before* claiming them. Should be:
    // ((0.1                    / 10_000)          * 100)               * 500         = 0.5 NTRN
    // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user3").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(500_000u128)
    );

    // Update the stake information for user3, acting as the proxy (set stake to 0)
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user3").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user3").into_string(),
        env.block.height,
        coin(0u128, "untrn"),
    );
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info_proxy.clone(),
        msg_update_stake,
    )
    .unwrap();

    // BLOCKS >>> 100
    env.block.height += 100;

    // user2 queries pending rewards *before* claiming them. Should be:
    // ((0.1                    / 10_000)          * 100)               * 1000         = 1 NTRN
    // +
    // ((0.1                    / 10_000)          * 100)               * 1000         = 1 NTRN
    // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user2").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(2_000_000u128)
    );

    // user3 queries pending rewards *before* claiming them. Should be:
    // ((0.1                    / 10_000)          * 100)               * 500         = 0.5 NTRN
    // +
    // ((0.1                    / 10_000)          * 100)               * 0           = 0 NTRN
    // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user3").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(500_000u128)
    );
}

/// Tests the following scenario:
///     (Yearly blocks: 10_000, APR: 0%)
///     1.  User4 stakes 1000
///     2.  100 blocks pass
///     3.  User queries the rewards (0 NTRN)
///     4.  User claims the rewards (0 NTRN, no BankSend)
#[test]
fn test_single_user_zero_annual_reward_rate_bps() {
    let mut deps = mock_dependencies();

    // Instantiate
    let mut env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let mut msg = default_init_msg(deps.api);
    msg.annual_reward_rate_bps = 0;
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // Ensure config is saved
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, deps.api.addr_make("owner"));
    assert_eq!(config.staking_info_proxy, deps.api.addr_make("proxy"));
    assert_eq!(config.dao_address, deps.api.addr_make("dao"));
    assert_eq!(config.annual_reward_rate_bps, 0);
    assert_eq!(config.blocks_per_year, 10_000);
    assert_eq!(config.staking_denom, "untrn");

    // Update the stake information from an unauthorized address
    let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
    let msg_update_stake = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user4").into_string(),
    };
    deps.querier.update_stake(
        deps.api.addr_make("user4").into_string(),
        env.block.height,
        coin(1_000_000_000, "untrn"),
    );
    let _res = execute(deps.as_mut(), env.clone(), info_proxy, msg_update_stake).unwrap();

    // BLOCKS >>> 100
    env.block.height += 100;

    // user4 queries pending rewards. Should be 0
    let query_msg = QueryMsg::Rewards {
        user: deps.api.addr_make("user4").into_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_resp.pending_rewards.amount, Uint128::new(0u128));

    // user4 claims
    let info_user4 = message_info(&deps.api.addr_make("user4"), &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), info_user4, claim_msg).unwrap();
    // Should have 0 BankMsg, since there is nothing to claim
    assert_eq!(res.messages.len(), 0);
}

/// A slashing event occurs that does not affect the user’s stake.
/// (This simulates the case where the user delegated to a validator that was not slashed.)
#[test]
fn test_slashing_no_effect() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();

    // Define addresses for owner, proxy (staking_info_proxy), DAO, and user1.
    let owner = deps.api.addr_make("owner");
    let proxy = deps.api.addr_make("proxy");
    let dao = deps.api.addr_make("dao");
    let user1 = deps.api.addr_make("user1");

    // Instantiate the contract.
    let instantiate_info = message_info(&owner, &[]);
    let instantiate_msg = InstantiateMsg {
        owner: owner.to_string(),
        dao_address: dao.to_string(),
        staking_info_proxy: proxy.to_string(),
        annual_reward_rate_bps: 1000, // 10% annual rate.
        blocks_per_year: 10_000,      // 10,000 blocks per year.
        staking_denom: "untrn".to_string(),
        security_address: deps.api.addr_make("security_address").into(),
    };
    let _res = instantiate(
        deps.as_mut(),
        env.clone(),
        instantiate_info,
        instantiate_msg,
    )
    .unwrap();

    // ---- STEP 1: Set an initial stake for user1.
    // At block 100, simulate that user1 has staked 1_000_000_000 units.
    env.block.height += 100;
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let proxy_info = message_info(&proxy, &[]);
    let update_msg = ExecuteMsg::UpdateStake {
        user: user1.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), update_msg).unwrap();

    // ---- STEP 2: Process a slashing event that does NOT change the user’s stake.
    // Advance to block 150.
    env.block.height += 50;
    // In this scenario the querier still returns the original stake.
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let slashing_msg = ExecuteMsg::Slashing {};
    let res = execute(
        deps.as_mut(),
        env.clone(),
        proxy_info.clone(),
        slashing_msg.clone(),
    )
    .unwrap();
    // Verify that the slashing event was recorded.
    assert!(res.attributes.iter().any(|attr| attr.value == "slashing"));

    // ---- STEP 3: Advance time and check rewards.
    // Advance to block 250.
    env.block.height += 100;
    // When no stake change occurred, rewards accumulate continuously.
    // Calculation:
    //   - From block 100 to 150: 50 blocks with stake 1_000_000_000 → 50 * (1_000_000_000 * 0.1/10_000)
    //       = 50 * 10,000 = 500,000.
    //   - From block 150 to 250: 100 blocks with stake 1_000_000_000 → 100 * 10,000 = 1,000,000.
    // Total expected pending rewards = 500,000 + 1,000,000 = 1,500,000.
    let query_msg = QueryMsg::Rewards {
        user: user1.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards.pending_rewards.amount, Uint128::new(1_500_000u128));

    // ---- STEP 4: User claims rewards.
    let user_info = message_info(&user1, &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), user_info, claim_msg).unwrap();
    // Expect one BankMsg message to be sent.
    assert_eq!(res.messages.len(), 1);

    // After claiming, pending rewards should be zero.
    let query_msg = QueryMsg::Rewards {
        user: user1.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_after: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_after.pending_rewards.amount, Uint128::zero());
}

/// Tests a single slashing event, making sure that the user gets full rewards for the period
/// before slashing and reduced rewards for the period after slashing.
#[test]
fn test_slashing_single_event() {
    // Create mock dependencies and environment.
    let mut deps = mock_dependencies();
    let mut env = mock_env();

    // Define addresses for the owner, proxy (which acts as the staking_info_proxy),
    // the DAO, and a sample user.
    let owner = deps.api.addr_make("owner");
    let proxy = deps.api.addr_make("proxy");
    let dao = deps.api.addr_make("dao");
    let user1 = deps.api.addr_make("user1");

    // Instantiate the contract.
    // (Adjust the InstantiateMsg fields as needed.)
    let instantiate_info = message_info(&owner, &[]);
    let instantiate_msg = InstantiateMsg {
        owner: owner.to_string(),
        dao_address: dao.to_string(),
        staking_info_proxy: proxy.to_string(),
        annual_reward_rate_bps: 1000, // e.g. 10% annual rate
        blocks_per_year: 10_000,      // e.g. 10,000 blocks per year
        staking_denom: "untrn".to_string(),
        security_address: deps.api.addr_make("security_address").into(),
    };
    let _res = instantiate(
        deps.as_mut(),
        env.clone(),
        instantiate_info,
        instantiate_msg,
    )
    .unwrap();

    // ----- STEP 1: Set an initial stake for user1 -----
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let proxy_info = message_info(&proxy, &[]);
    let update_stake_msg = ExecuteMsg::UpdateStake {
        user: user1.to_string(),
    };
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        proxy_info.clone(),
        update_stake_msg,
    )
    .unwrap();

    // Advance the block height by 100 blocks.
    env.block.height += 100;
    // (At this point, if no slashing event occurred, user1’s rewards would be:
    //   reward = stake * rate_per_block * num_blocks
    // where rate_per_block = (annual_rate / blocks_per_year)
    // For these parameters, after 100 blocks, the expected reward would be 1_000_000 units.)

    // ----- STEP 2: Simulate a slashing event -----
    // Advance block height by 50 blocks so that the slashing event occurs at a new height.
    env.block.height += 50; // Now, env.block.height is (initial + 150)
    let slashing_height = env.block.height;

    // At the moment of the slashing event, simulate that user1’s stake has been slashed,
    // reducing from 1_000_000_000 to 500_000_000 units.
    deps.querier.update_stake(
        user1.to_string(),
        slashing_height,
        coin(500_000_000u128, "untrn"),
    );

    // Execute the slashing event (only allowed by the proxy).
    let slashing_msg = ExecuteMsg::Slashing {};
    let res = execute(
        deps.as_mut(),
        env.clone(),
        proxy_info.clone(),
        slashing_msg.clone(),
    )
    .unwrap();
    // Verify that the slashing event was recorded by checking one of the attributes.
    assert!(res.attributes.iter().any(|attr| attr.value == "slashing"));

    // Verify that sending a duplicate slashing event at the same block fails.
    let res = execute(deps.as_mut(), env.clone(), proxy_info.clone(), slashing_msg).unwrap();
    assert!(res.attributes.iter().any(|attr| attr.value == "ignored"));

    // ----- STEP 3: Accrue additional rewards after the slashing event -----
    // Advance the block height by 100 blocks.
    env.block.height += 100;
    // New height: (initial + 250)
    //
    // Expected rewards breakdown:
    //   * From the initial update (at block  ... let's call it T₀) to the slashing event (at block 150):
    //       stake = 1_000_000_000 units for 150 blocks.
    //       Reward = 1_000_000_000 * (annual_rate/blocks_per_year) * 150.
    //       With annual_rate_bps=1000 → annual_rate=0.1 and blocks_per_year=10_000,
    //       rate_per_block = 0.1/10_000 = 0.00001.
    //       Thus, reward_pre_slash = 1_000_000_000 * 0.00001 * 150 = 1_500_000.
    //
    //   * After the slashing event (from block 150 to block 250):
    //       stake = 500_000_000 units for 100 blocks.
    //       Reward = 500_000_000 * 0.00001 * 100 = 500_000.
    //
    // Total expected pending rewards = 1_500_000 + 500_000 = 2_000_000.

    let query_rewards_msg = QueryMsg::Rewards {
        user: user1.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_rewards_msg).unwrap();
    let rewards_resp: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(
        rewards_resp.pending_rewards.amount,
        Uint128::new(2_000_000u128)
    );

    // ----- STEP 4: User claims rewards -----
    let user1_info = message_info(&user1, &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), user1_info, claim_msg).unwrap();
    // Verify that a BankMsg is generated to send the rewards.
    assert_eq!(res.messages.len(), 1);

    // Finally, query rewards again to confirm that pending rewards are now zero.
    let query_rewards_msg = QueryMsg::Rewards {
        user: user1.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_rewards_msg).unwrap();
    let rewards_after_claim: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_after_claim.pending_rewards.amount, Uint128::zero());

    // remove state entity for the slashing call to fail
    STATE.remove(&mut deps.storage);
    // Execute the slashing event (only allowed by the proxy).
    let slashing_msg = ExecuteMsg::Slashing {};
    execute(
        deps.as_mut(),
        env.clone(),
        proxy_info.clone(),
        slashing_msg.clone(),
    )
    .unwrap(); // the contract should raise an error
               // but instead the contract must be paused after an errored call
    assert!(PAUSED.load(&deps.storage).unwrap());
}

/// Two slashing events occur before the user queries and claims rewards.
#[test]
fn test_multiple_slashing_events() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();

    let owner = deps.api.addr_make("owner");
    let proxy = deps.api.addr_make("proxy");
    let dao = deps.api.addr_make("dao");
    let user1 = deps.api.addr_make("user1");

    // Instantiate the contract.
    let instantiate_info = message_info(&owner, &[]);
    let instantiate_msg = InstantiateMsg {
        owner: owner.to_string(),
        dao_address: dao.to_string(),
        staking_info_proxy: proxy.to_string(),
        annual_reward_rate_bps: 1000,
        blocks_per_year: 10_000,
        staking_denom: "untrn".to_string(),
        security_address: deps.api.addr_make("security_address").into(),
    };
    let _ = instantiate(
        deps.as_mut(),
        env.clone(),
        instantiate_info,
        instantiate_msg,
    )
    .unwrap();

    // ---- STEP 1: Set initial stake at block 100.
    env.block.height += 100;
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    let proxy_info = message_info(&proxy, &[]);
    let update_msg = ExecuteMsg::UpdateStake {
        user: user1.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), update_msg).unwrap();

    // ---- STEP 2: First slashing event at block 150.
    // For this event, simulate a 50% cut: stake goes from 1_000_000_000 → 500_000_000.
    env.block.height += 50;
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(500_000_000u128, "untrn"),
    );
    let slashing_msg = ExecuteMsg::Slashing {};
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), slashing_msg).unwrap();

    // ---- STEP 3: Second slashing event at block 200.
    // Now simulate another 50% cut: stake goes from 500_000_000 → 250_000_000.
    env.block.height += 50;
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(250_000_000u128, "untrn"),
    );
    let slashing_msg = ExecuteMsg::Slashing {};
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), slashing_msg).unwrap();

    // ---- STEP 4: Advance time and check rewards.
    // Advance to block 250.
    env.block.height += 50;
    // Expected rewards breakdown:
    //   - From block 100 to 150 (50 blocks) with stake 1_000_000_000:
    //         50 * (1_000_000_000 * 0.1/10_000) = 50 * 10,000 = 500,000.
    //   - From block 150 to 200 (50 blocks) with stake 500_000_000:
    //         50 * (500_000_000 * 0.1/10_000) = 50 * 5,000 = 250,000.
    //   - From block 200 to 250 (50 blocks) with stake 250_000_000:
    //         50 * (250_000_000 * 0.1/10_000) = 50 * 2,500 = 125,000.
    // Total expected pending rewards = 500,000 + 250,000 + 125,000 = 875,000.
    let query_msg = QueryMsg::Rewards {
        user: user1.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards.pending_rewards.amount, Uint128::new(875_000u128));

    // ---- STEP 5: User claims rewards.
    let user_info = message_info(&user1, &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), user_info, claim_msg).unwrap();
    assert_eq!(res.messages.len(), 1);

    // Verify that rewards are now 0.
    let query_msg = QueryMsg::Rewards {
        user: user1.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_after: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_after.pending_rewards.amount, Uint128::zero());
}

/// A corner case where the user updates the stake at block N (e.g. delegates an additional 500_000_000 tokens)
/// but then, still in block N, a slashing event is processed that cuts the user’s stake in half.
#[test]
fn test_update_and_slash_same_block() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();

    let owner = deps.api.addr_make("owner");
    let proxy = deps.api.addr_make("proxy");
    let dao = deps.api.addr_make("dao");
    let user1 = deps.api.addr_make("user1");

    // Instantiate the contract.
    let instantiate_info = message_info(&owner, &[]);
    let instantiate_msg = InstantiateMsg {
        owner: owner.to_string(),
        dao_address: dao.to_string(),
        staking_info_proxy: proxy.to_string(),
        annual_reward_rate_bps: 1000,
        blocks_per_year: 10_000,
        staking_denom: "untrn".to_string(),
        security_address: deps.api.addr_make("security_address").into(),
    };
    let _ = instantiate(
        deps.as_mut(),
        env.clone(),
        instantiate_info,
        instantiate_msg,
    )
    .unwrap();

    // ---- STEP 1: Set an initial stake for user1 in an earlier block.
    // Let’s say at block 200, the user’s stake is 500_000_000.
    env.block.height += 200;
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(500_000_000u128, "untrn"),
    );
    let proxy_info = message_info(&proxy, &[]);
    let update_msg = ExecuteMsg::UpdateStake {
        user: user1.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), update_msg).unwrap();

    // ---- STEP 2: At block 300 the user delegates an additional 500_000_000 tokens.
    // So the staking_info_proxy should return 1_000_000_000 at first.
    env.block.height += 100;
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(1_000_000_000u128, "untrn"),
    );
    // Process the update_stake message. This will add 100 * (500_000_000 * (0.1/10_000)) = 500_000
    // to user's pending rewards.
    let update_msg = ExecuteMsg::UpdateStake {
        user: user1.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), update_msg).unwrap();

    // At the same block (300) the endblocker processes a slashing event.
    // This slashing event should now cut the user’s stake in half.
    // We simulate this by updating the querier to return 500_000_000 for the slashing event.
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(500_000_000u128, "untrn"),
    );
    let slashing_msg = ExecuteMsg::Slashing {};
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), slashing_msg).unwrap();

    // ---- STEP 3: Advance time so that rewards accrue after block 300.
    // From block 300 to block 350, the stake used for rewards should be the slashed stake: 500_000_000.
    env.block.height += 50;
    // Expected rewards for 50 blocks:
    //   50 * (500_000_000 * (0.1/10_000)) = 50 * (500_000_000 * 0.00001) = 50 * 5,000 = 250,000.
    // Combined with the still pending 500_000 rewards from blocks 200 to 300, total pending rewards
    // now should be 750_000.
    let query_msg = QueryMsg::Rewards {
        user: user1.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards.pending_rewards.amount, Uint128::new(750_000u128));

    // ---- STEP 4: User claims rewards.
    let user_info = message_info(&user1, &[]);
    let claim_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let res = execute(deps.as_mut(), env.clone(), user_info, claim_msg).unwrap();
    assert_eq!(res.messages.len(), 1);

    // After claiming, pending rewards should be zero.
    let query_msg = QueryMsg::Rewards {
        user: user1.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let rewards_after: RewardsResponse = cosmwasm_std::from_json(bin).unwrap();
    assert_eq!(rewards_after.pending_rewards.amount, Uint128::zero());
}

// Test scenario where user1 and user2 accrues funds.
// - user1 gets slashed.
// - user2 claims funds later.
// - then user1 claims later. user1 claim amount should account slashing correctly
//      and do not break when global index change after user2 claim
#[test]
fn test_two_users_with_slashing() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.block.height = 0;

    let owner = deps.api.addr_make("owner");
    let proxy = deps.api.addr_make("proxy");
    let dao = deps.api.addr_make("dao");
    let user1 = deps.api.addr_make("user1");
    let user2 = deps.api.addr_make("user2");

    let user1_info = message_info(&user1, &[]);
    let user2_info = message_info(&user2, &[]);
    let proxy_info = message_info(&proxy, &[]);

    let user_stake_before_slashing = 500_000_000u128;
    let user_stake_after_slashing = 250_000_000u128;

    // Instantiate the contract.
    let instantiate_info = message_info(&owner, &[]);
    let instantiate_msg = InstantiateMsg {
        owner: owner.to_string(),
        dao_address: dao.to_string(),
        staking_info_proxy: proxy.to_string(),
        annual_reward_rate_bps: 1000, // 10%
        blocks_per_year: 100,
        staking_denom: "untrn".to_string(),
        security_address: deps.api.addr_make("security_address").into(),
    };
    let _ = instantiate(
        deps.as_mut(),
        env.clone(),
        instantiate_info,
        instantiate_msg,
    )
    .unwrap();

    // initialize user1 and user2 with 500_000_000 stake
    deps.querier.update_stake(
        user1.to_string(),
        env.block.height,
        coin(user_stake_before_slashing, "untrn"),
    );
    let update_msg_1 = ExecuteMsg::UpdateStake {
        user: user1.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), update_msg_1).unwrap();

    deps.querier.update_stake(
        user2.to_string(),
        env.block.height,
        coin(user_stake_before_slashing, "untrn"),
    );
    let update_msg_2 = ExecuteMsg::UpdateStake {
        user: user2.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), update_msg_2).unwrap();

    // pass a year
    env.block.height += 100;

    // simulate slashing user2
    let slashing_msg = ExecuteMsg::Slashing {};
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), slashing_msg).unwrap();
    // slashed 50% of user1's stake
    deps.querier.update_stake(
        user2.to_string(),
        env.block.height,
        coin(user_stake_after_slashing, "untrn"),
    );

    // pass a year
    env.block.height += 100;

    // claim for user1
    let claim_user1_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let update_user1_res = execute(
        deps.as_mut(),
        env.clone(),
        user1_info.clone(),
        claim_user1_msg,
    )
    .unwrap();
    let user1_claimed = unwrap_send_amount_from_update_stake(update_user1_res);

    // claim for user2
    let claim_user2_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let update_user2_res = execute(
        deps.as_mut(),
        env.clone(),
        user2_info.clone(),
        claim_user2_msg,
    )
    .unwrap();
    let user2_claimed = unwrap_send_amount_from_update_stake(update_user2_res);

    // for user 2 slashing amount should be:
    let expected_user1_claimed = user_stake_before_slashing as f64 * 0.2;
    // (user2_stake_before_slashing * year_rewards) + (user_2_stake_after_slashing * year_rewards)
    let expected_user2_claimed =
        user_stake_before_slashing as f64 * 0.1 + user_stake_after_slashing as f64 * 0.1;

    // amount of claimed user1 should be less since the slashing
    assert_eq!(user1_claimed.u128() as f64, expected_user1_claimed);
    assert_eq!(user2_claimed.u128() as f64, expected_user2_claimed);
}

// - stake with user 500ntrn
// - slash event (not this user, just slash) year later
// - update config year later and do not even slash the user (lets imagine other users get slashed)
// - rewards should correctly accrue in regard to new config values
#[test]
fn test_user_with_slashing_and_config_change() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.block.height = 0;

    let owner = deps.api.addr_make("owner");
    let proxy = deps.api.addr_make("proxy");
    let dao = deps.api.addr_make("dao");
    let user = deps.api.addr_make("user");

    let owner_info = message_info(&owner, &[]);
    let user_info = message_info(&user, &[]);
    let proxy_info = message_info(&proxy, &[]);

    let user_stake = 500_000_000u128;

    // Instantiate the contract.
    let instantiate_info = message_info(&owner, &[]);
    let instantiate_msg = InstantiateMsg {
        owner: owner.to_string(),
        dao_address: dao.to_string(),
        staking_info_proxy: proxy.to_string(),
        annual_reward_rate_bps: 1000, // 10%
        blocks_per_year: 100,
        staking_denom: "untrn".to_string(),
        security_address: deps.api.addr_make("security_address").into(),
    };
    let _ = instantiate(
        deps.as_mut(),
        env.clone(),
        instantiate_info,
        instantiate_msg,
    )
    .unwrap();

    // initialize user1 and user2 with 500_000_000 stake
    deps.querier.update_stake(
        user.to_string(),
        env.block.height,
        coin(user_stake, "untrn"),
    );
    let update_msg_1 = ExecuteMsg::UpdateStake {
        user: user.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), update_msg_1).unwrap();

    // pass a year
    env.block.height += 100;

    // simulate slashing user2
    let slashing_msg = ExecuteMsg::Slashing {};
    let _ = execute(deps.as_mut(), env.clone(), proxy_info.clone(), slashing_msg).unwrap();

    // pass a year
    env.block.height += 100;

    let update_config_msg = ExecuteMsg::UpdateConfig {
        owner: None,
        annual_reward_rate_bps: Some(5000), // 50%
        blocks_per_year: None,
        staking_info_proxy: None,
        staking_denom: None,
        security_address: None,
    };
    let _ = execute(
        deps.as_mut(),
        env.clone(),
        owner_info.clone(),
        update_config_msg,
    )
    .unwrap();

    // pass a year
    env.block.height += 100;

    // claim for user1
    let claim_user1_msg = ExecuteMsg::ClaimRewards { to_address: None };
    let update_user_res = execute(
        deps.as_mut(),
        env.clone(),
        user_info.clone(),
        claim_user1_msg,
    )
    .unwrap();
    let actual_user_claimed = unwrap_send_amount_from_update_stake(update_user_res);

    // 3 years passed from them
    // - 2 years with 10% annual rewards
    // - 1 year with 50% annual rewards
    // total rewards should be 70% of stake
    let expected_user_claimed = 0.2 * user_stake as f64 + 0.5 * user_stake as f64;

    // amount of claimed user1 should be less since the slashing
    assert_eq!(actual_user_claimed.u128() as f64, expected_user_claimed);
}

// helpers
fn unwrap_send_amount_from_update_stake(res: Response) -> Uint128 {
    res.messages
        .into_iter()
        .find_map(|m| match m.msg {
            CosmosMsg::Bank(BankMsg::Send {
                to_address: _,
                amount,
            }) => return Some(amount.first().unwrap().amount),
            _ => None,
        })
        .unwrap()
}
