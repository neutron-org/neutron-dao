use crate::contract::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{CONFIG, PROVIDERS};
use crate::testing::mock_querier::{
    mock_dependencies, PROVIDER1, PROVIDER2, STAKING_REWARDS_CONTRACT,
};
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::{
    testing::{message_info, mock_env},
    Addr, Order,
};

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
    assert!(res_2.is_err());

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
        providers: vec![PROVIDER1.to_string(), PROVIDER2.to_string()],
    };

    let info2 = message_info(&deps.api.addr_make("stranger"), &[]);
    let res_2 = execute(deps.as_mut(), env.clone(), info2, update_msg.clone());
    assert!(res_2.is_err());

    // Authorized update
    let res_3 = execute(deps.as_mut(), env.clone(), info.clone(), update_msg);
    assert!(res_3.is_ok());

    let providers: Vec<Addr> = PROVIDERS
        .keys(&deps.storage, None, None, Order::Descending)
        .map(|k| k.unwrap())
        .collect();
    assert_eq!(
        providers,
        vec![Addr::unchecked(PROVIDER1), Addr::unchecked(PROVIDER2)]
    );
}
