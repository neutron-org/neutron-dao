use crate::contract::{execute, instantiate, query};
use crate::error::ContractError::Unauthorized;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CONFIG, PROVIDERS};
use crate::testing::mock_querier::{
    mock_dependencies, PROVIDER1, PROVIDER2, PROVIDER3, STAKING_REWARDS_CONTRACT,
};
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::{
    from_json,
    testing::{message_info, mock_env},
    to_json_binary, Addr, Coin, Order, SubMsg, Uint128, WasmMsg,
};
use neutron_staking_rewards::msg::ExecuteMsg::UpdateStake as RewardsMsgUpdateStake;

// Helper to create a default instantiate message
fn default_init_msg(api: MockApi) -> InstantiateMsg {
    InstantiateMsg {
        owner: api.addr_make("owner").into(),
        staking_rewards: None,
        staking_denom: "untrn".to_string(),
        providers: vec![],
    }
}

/// Tests the following scenario:
///     1.  A non-authorized address tries to update the user's stake (error)
///     2.  An authorized address tries to update config's staking_rewards contract
#[test]
fn test_update_config() {
    let mut deps = mock_dependencies();

    // Instantiate
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, info.sender);

    // Unauthorized update
    let update_msg = ExecuteMsg::UpdateConfig {
        owner: None,
        staking_rewards: Some(STAKING_REWARDS_CONTRACT.to_string()),
        staking_denom: None,
    };

    let info2 = message_info(&deps.api.addr_make("stranger"), &[]);
    let res_2 = execute(deps.as_mut(), env.clone(), info2, update_msg.clone());
    assert_eq!(res_2.err().unwrap(), Unauthorized {});

    // Authorized update
    let res_3 = execute(deps.as_mut(), env.clone(), info.clone(), update_msg);
    assert!(res_3.is_ok());

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, info.sender);
    assert_eq!(config.staking_denom, "untrn");
    assert_eq!(
        config.staking_rewards.map(|s| s.to_string()),
        Some(STAKING_REWARDS_CONTRACT.to_string())
    );
}

/// Tests the following scenario:
///     1.  A non-authorized address tries to update providers (error)
///     2.  An authorized address tries to update providers
#[test]
fn test_update_providers() {
    let mut deps = mock_dependencies();

    // Instantiate
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = default_init_msg(deps.api);
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, info.sender);

    // Unauthorized update
    let update_msg = ExecuteMsg::UpdateProviders {
        providers: vec![
            deps.api.addr_make("provider1").to_string(),
            deps.api.addr_make("provider2").to_string(),
        ],
    };

    let info2 = message_info(&deps.api.addr_make("stranger"), &[]);
    let res_2 = execute(deps.as_mut(), env.clone(), info2, update_msg.clone());
    assert_eq!(res_2.err().unwrap(), Unauthorized {});

    // Authorized update
    let res_3 = execute(deps.as_mut(), env.clone(), info.clone(), update_msg);
    assert!(res_3.is_ok());

    let providers: Vec<Addr> = PROVIDERS
        .keys(&deps.storage, None, None, Order::Ascending)
        .map(|k| k.unwrap())
        .collect();
    assert_eq!(
        providers,
        vec![
            deps.api.addr_make("provider1"),
            deps.api.addr_make("provider2")
        ]
    );
}

/// Tests the following scenario:
///     1.  A non-authorized address tries to proxy stake update (error)
///     2.  An authorized address tries to proxy stake update (check correct RewardsMsgUpdateStake is created)
#[test]
fn test_update_stake() {
    let mut deps = mock_dependencies();

    // Instantiate
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = InstantiateMsg {
        owner: deps.api.addr_make("owner").into(),
        staking_rewards: Some(STAKING_REWARDS_CONTRACT.to_string()),
        staking_denom: "untrn".to_string(),
        providers: vec![
            deps.api.addr_make("provider1").to_string(),
            deps.api.addr_make("provider2").to_string(),
        ],
    };
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let stranger_provider_info = message_info(&deps.api.addr_make("stranger"), &[]);
    let msg = ExecuteMsg::UpdateStake {
        user: deps.api.addr_make("user1").into(),
    };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        stranger_provider_info,
        msg.clone(),
    );
    assert_eq!(res.err().unwrap(), Unauthorized {});

    let provider_info = message_info(&deps.api.addr_make("provider1"), &[]);
    let res = execute(deps.as_mut(), env.clone(), provider_info, msg.clone());
    assert!(res.is_ok());
    let resp = res.unwrap();

    let expected = WasmMsg::Execute {
        contract_addr: STAKING_REWARDS_CONTRACT.to_string(),
        msg: to_json_binary(&RewardsMsgUpdateStake {
            user: deps.api.addr_make("user1").to_string(),
        })
        .unwrap(),
        funds: vec![],
    };
    assert_eq!(resp.messages, vec![SubMsg::reply_never(expected)])
}

/// Tests the following scenario:
///     1. Query with no providers set
///     2. Query with one provider
///     3. Query with multiple providers, some of them return error
///     4. Query with multiple providers, some of them return non staking denom
#[test]
fn test_query_stake_query() {
    let mut deps = mock_dependencies();

    // Instantiate
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("owner"), &[]);
    let msg = InstantiateMsg {
        owner: deps.api.addr_make("owner").into(),
        staking_rewards: Some(STAKING_REWARDS_CONTRACT.to_string()),
        staking_denom: "untrn".to_string(),
        providers: vec![],
    };
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let query_msg = QueryMsg::StakeQuery {
        user: deps.api.addr_make("user").to_string(),
        height: None,
    };

    // No providers returns zero in any case
    let q1 = query(deps.as_ref(), env.clone(), query_msg.clone()).unwrap();
    let c1: Coin = from_json(q1).unwrap();
    assert_eq!(c1.amount, Uint128::zero());

    // Set providers
    let set_msg = ExecuteMsg::UpdateProviders {
        providers: vec![PROVIDER1.to_string(), PROVIDER2.to_string()],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), set_msg.clone());
    assert!(res.is_ok());

    // Check that has some result
    let q2 = query(deps.as_ref(), env.clone(), query_msg.clone()).unwrap();
    let c2: Coin = from_json(q2).unwrap();
    assert_eq!(c2.amount, Uint128::new(300));

    // Set providers with one that returns Err
    let set_msg = ExecuteMsg::UpdateProviders {
        providers: vec![
            PROVIDER1.to_string(),
            PROVIDER2.to_string(),
            PROVIDER3.to_string(),
        ],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), set_msg.clone());
    assert!(res.is_ok());

    let q3 = query(deps.as_ref(), env.clone(), query_msg.clone()).unwrap();
    let c3: Coin = from_json(q3).unwrap();
    assert_eq!(c3.amount, Uint128::new(300));
}
