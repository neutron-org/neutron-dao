#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RewardsResponse};
    use crate::state::CONFIG;
    use crate::testing::mock_querier::mock_dependencies;
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{
        coin,
        testing::{message_info, mock_env},
        Uint128,
    };

    // Helper to create a default instantiate message
    fn default_init_msg(api: MockApi) -> InstantiateMsg {
        InstantiateMsg {
            owner: api.addr_make("owner").into(),
            annual_reward_rate_bps: 1000, // 10%
            blocks_per_year: 10_000,
            dao_address: api.addr_make("dao").into(),
            staking_info_proxy: api.addr_make("proxy").into(),
            staking_denom: "untrn".to_string(),
        }
    }

    /// Tests the following scenario:
    ///     1.  A non-authorized address tries to update the user's stake (error)
    ///     2.  An authorized address tries to update stake for the DAO address (error)
    #[test]
    fn test_update_stake() {
        let mut deps = mock_dependencies();

        // Instantiate
        let env = mock_env();
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
        deps.querier.user_balances.insert(
            deps.api.addr_make("user0").into_string(),
            coin(1_000_000_000, "untrn"),
        );
        let res = execute(deps.as_mut(), env.clone(), info_not_proxy, msg_update_stake);
        assert_eq!(res.err().unwrap(), ContractError::Unauthorized {});

        // Update the stake information for the DAO
        let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
        let msg_update_stake = ExecuteMsg::UpdateStake {
            user: deps.api.addr_make("dao").into_string(),
        };
        deps.querier.user_balances.insert(
            deps.api.addr_make("user0").into_string(),
            coin(1_000_000_000, "untrn"),
        );
        let res = execute(deps.as_mut(), env.clone(), info_proxy, msg_update_stake);
        assert_eq!(
            res.err().unwrap(),
            ContractError::DaoStakeChangeNotTracked {}
        )
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
        deps.querier.user_balances.insert(
            deps.api.addr_make("user1").into_string(),
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(
            rewards_resp.pending_rewards.amount,
            Uint128::new(2_000_000u128)
        );

        // user1 claims
        let info_user1 = message_info(&deps.api.addr_make("user1"), &[]);
        let claim_msg = ExecuteMsg::ClaimRewards {};
        let res = execute(deps.as_mut(), env.clone(), info_user1, claim_msg).unwrap();
        // Should have 1 BankMsg to send the user’s rewards
        assert_eq!(res.messages.len(), 1);

        // user1 queries rewards *after* claiming them, should be 0
        let query_msg = QueryMsg::Rewards {
            user: deps.api.addr_make("user1").into_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(rewards_resp.pending_rewards.amount, Uint128::new(0));

        // ------------------------STEP 3------------------------------------------

        // BLOCKS >>> 100
        env.block.height += 100;

        // Update the stake information, reducing user1's stake by 50%
        let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
        let msg_update_stake = ExecuteMsg::UpdateStake {
            user: deps.api.addr_make("user1").into_string(),
        };
        deps.querier.user_balances.insert(
            deps.api.addr_make("user1").into_string(),
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(
            rewards_resp.pending_rewards.amount,
            Uint128::new(1_500_000u128)
        );
        // user1 claims
        let info_user1 = message_info(&deps.api.addr_make("user1"), &[]);
        let claim_msg = ExecuteMsg::ClaimRewards {};
        let res = execute(deps.as_mut(), env.clone(), info_user1, claim_msg).unwrap();
        // Should have 1 BankMsg to send the user’s rewards
        assert_eq!(res.messages.len(), 1);

        // ------------------------STEP 4------------------------------------------

        // Update the stake information, reducing user1's stake to 0
        let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
        let msg_update_stake = ExecuteMsg::UpdateStake {
            user: deps.api.addr_make("user1").into_string(),
        };
        deps.querier
            .user_balances
            .insert(deps.api.addr_make("user1").into_string(), coin(0, "untrn"));
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(rewards_resp.pending_rewards.amount, Uint128::new(0u128));
        // user1 claims
        let info_user1 = message_info(&deps.api.addr_make("user1"), &[]);
        let claim_msg = ExecuteMsg::ClaimRewards {};
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
    ///     1. 100 blocks pass
    ///     2. User2 queries the rewards (0 NTRN)
    ///     3. User3 queries the rewards (0 NTRN)
    ///     ------------------------STEP 3------------------------------------------
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
        deps.querier.user_balances.insert(
            deps.api.addr_make("user2").into_string(),
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
        deps.querier.user_balances.insert(
            deps.api.addr_make("user3").into_string(),
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(
            rewards_resp.pending_rewards.amount,
            Uint128::new(500_000u128)
        );

        // user2 claims
        let info_user2 = message_info(&deps.api.addr_make("user2"), &[]);
        let claim_msg = ExecuteMsg::ClaimRewards {};
        let res = execute(deps.as_mut(), env.clone(), info_user2, claim_msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // user3 claims
        let info_user3 = message_info(&deps.api.addr_make("user3"), &[]);
        let claim_msg = ExecuteMsg::ClaimRewards {};
        let res = execute(deps.as_mut(), env.clone(), info_user3, claim_msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // user2 queries pending rewards *after* claiming them, should be 0
        let query_msg = QueryMsg::Rewards {
            user: deps.api.addr_make("user2").into_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(
            rewards_resp.pending_rewards.amount,
            Uint128::new(0u128)
        );

        // user3 queries pending rewards *after* claiming them, should be 0
        let query_msg = QueryMsg::Rewards {
            user: deps.api.addr_make("user3").into_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(
            rewards_resp.pending_rewards.amount,
            Uint128::new(0u128)
        );

        // ------------------------STEP 2------------------------------------------

        // BLOCKS >>> 100
        env.block.height += 100;

        // user2 queries pending rewards *after* claiming them, should be 0
        let query_msg = QueryMsg::Rewards {
            user: deps.api.addr_make("user2").into_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(
            rewards_resp.pending_rewards.amount,
            Uint128::new(0u128)
        );

        // user3 queries pending rewards *after* claiming them, should be 0
        let query_msg = QueryMsg::Rewards {
            user: deps.api.addr_make("user3").into_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(
            rewards_resp.pending_rewards.amount,
            Uint128::new(0u128)
        );

        //     ------------------------STEP 3------------------------------------------
        // Update the stake information for user2
        let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
        let msg_update_stake = ExecuteMsg::UpdateStake {
            user: deps.api.addr_make("user2").into_string(),
        };
        deps.querier.user_balances.insert(
            deps.api.addr_make("user2").into_string(),
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
        deps.querier.user_balances.insert(
            deps.api.addr_make("user3").into_string(),
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(
            rewards_resp.pending_rewards.amount,
            Uint128::new(500_000u128)
        );

        // Update the stake information for user3, acting as the proxy (set stake to 0)
        let info_proxy = message_info(&deps.api.addr_make("proxy"), &[]);
        let msg_update_stake = ExecuteMsg::UpdateStake {
            user: deps.api.addr_make("user3").into_string(),
        };
        deps.querier.user_balances.insert(
            deps.api.addr_make("user3").into_string(),
            coin(0u128, "untrn"),
        );
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info_proxy.clone(),
            msg_update_stake,
        )
            .unwrap();

        // user2 queries pending rewards *before* claiming them. Should be:
        // ((0.1                    / 10_000)          * 100)               * 1000         = 1 NTRN
        // +
        // ((0.1                    / 10_000)          * 100)               * 1000         = 1 NTRN
        // ((annual_reward_rate_bps / blocks_per_year) * num_blocks_passed) * user_stake
        let query_msg = QueryMsg::Rewards {
            user: deps.api.addr_make("user2").into_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
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
        deps.querier.user_balances.insert(
            deps.api.addr_make("user4").into_string(),
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
        let rewards_resp: RewardsResponse = cosmwasm_std::from_json(&bin).unwrap();
        assert_eq!(rewards_resp.pending_rewards.amount, Uint128::new(0u128));

        // user4 claims
        let info_user4 = message_info(&deps.api.addr_make("user4"), &[]);
        let claim_msg = ExecuteMsg::ClaimRewards {};
        let res = execute(deps.as_mut(), env.clone(), info_user4, claim_msg).unwrap();
        // Should have 0 BankMsg, since there is nothing to claim
        assert_eq!(res.messages.len(), 0);
    }
}
