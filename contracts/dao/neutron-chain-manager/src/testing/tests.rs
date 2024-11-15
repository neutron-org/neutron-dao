use crate::contract::{
    execute_add_strategy, execute_execute_messages, execute_remove_strategy, instantiate,
};
use crate::error::ContractError::{InvalidDemotion, Unauthorized};
use crate::msg::Permission::{
    CronPermission, ParamChangePermission, UpdateCCVParamsPermission, UpdateCronParamsPermission,
    UpdateDexParamsPermission, UpdateDynamicfeesParamsPermission, UpdateGlobalfeeParamsPermission,
    UpdateTokenfactoryParamsPermission,
};
use crate::msg::{
    CCVUpdateParamsPermission, CronPermission as CronPermissionType, CronUpdateParamsPermission,
    DexUpdateParamsPermission, DynamicFeesUpdateParamsPermission, GlobalfeeUpdateParamsPermission,
    InstantiateMsg, ParamChangePermission as ParamChangePermissionType, ParamPermission,
    StrategyMsg, TokenfactoryUpdateParamsPermission,
};
use crate::testing::mock_querier::{
    consumer_params_to_update, default_consumer_params, mock_dependencies,
};
use cosmwasm_std::testing::{message_info, mock_env};
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Uint128};
use neutron_sdk::bindings::msg::{
    AdminProposal, NeutronMsg, ParamChange, ParamChangeProposal, ProposalExecuteMessage,
};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();
}

#[test]
fn test_add_strategy() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    // Scenario 1: An ALLOW_ALL strategy is added for a new address (passes).
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowAll,
    )
    .unwrap();

    // Scenario 2: An ALLOW_ONLY strategy is added for a new address (passes).
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr2".to_string()),
        StrategyMsg::AllowOnly(vec![CronPermission(CronPermissionType {
            add_schedule: true,
            remove_schedule: true,
        })]),
    )
    .unwrap();
}

/// An ALLOW_ALL strategy is added for an existing ALLOW_ONLY address (passes,
/// the promoted address can make privileged actions).
#[test]
fn test_add_strategy_promotion() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr2".to_string()),
        StrategyMsg::AllowOnly(vec![CronPermission(CronPermissionType {
            add_schedule: true,
            remove_schedule: true,
        })]),
    )
    .unwrap();

    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr2".to_string()),
        StrategyMsg::AllowAll,
    )
    .unwrap();
    let info = message_info(&Addr::unchecked("addr2"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr3".to_string()),
        StrategyMsg::AllowAll,
    )
    .unwrap();
}

/// An ALLOW_ONLY strategy is added for one of the existing ALLOW_ALL address
/// (passes, the demoted address can not make privileged actions).
#[test]
fn test_add_strategy_demotion() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowAll,
    )
    .unwrap();
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![CronPermission(CronPermissionType {
            add_schedule: true,
            remove_schedule: true,
        })]),
    )
    .unwrap();
    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_add_strategy(
        deps.as_mut(),
        info,
        Addr::unchecked("addr2".to_string()),
        StrategyMsg::AllowAll,
    )
    .unwrap_err();
    assert_eq!(err, Unauthorized {})
}

/// An ALLOW_ONLY strategy is added for the only existing ALLOW_ALL address
/// (fails).
#[test]
fn test_add_strategy_invalid_demotion() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let err = execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("neutron_dao_address".to_string()),
        StrategyMsg::AllowOnly(vec![CronPermission(CronPermissionType {
            add_schedule: true,
            remove_schedule: true,
        })]),
    )
    .unwrap_err();
    assert_eq!(err, InvalidDemotion {});
}

#[test]
fn test_remove_strategy() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowAll,
    )
    .unwrap();
    execute_remove_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_add_strategy(
        deps.as_mut(),
        info,
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowAll,
    )
    .unwrap_err();
    assert_eq!(err, Unauthorized {})
}

#[test]
fn test_remove_strategy_invalid_demotion() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let err = execute_remove_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("neutron_dao_address".to_string()),
    )
    .unwrap_err();
    assert_eq!(err, InvalidDemotion {});
}

/// Checks that if you have permissions, you can change both parameters of the cron
/// module (new style parameter changes). NOTE: this does not check that the
/// parameters have actually been changed.
#[test]
pub fn test_execute_execute_message_update_params_cron_authorized() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.cron.MsgUpdateParams",
            "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
            "params": {"security_address": "addr1", "limit": 16}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCronParamsPermission(
            CronUpdateParamsPermission {
                security_address: true,
                limit: true,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap();
}

/// Checks that unsupported message types inside a ProposalExecuteMessage are not
/// executed.
#[test]
pub fn test_execute_execute_message_unsupported_message_type_unauthorized() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.cron.MsgUnsupported",
            "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
            "params": {"security_address": "addr1", "limit": 16}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCronParamsPermission(
            CronUpdateParamsPermission {
                security_address: true,
                limit: true,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {})
}

/// Checks that you can't check the limit if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_cron_unauthorized_limit() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.cron.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"security_address": "neutron_dao_address", "limit": 16}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCronParamsPermission(
            CronUpdateParamsPermission {
                security_address: true,
                limit: false,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {})
}

/// Checks that you can't check the security_address if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_cron_unauthorized_security_address() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.cron.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"security_address": "addr1", "limit": 10}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCronParamsPermission(
            CronUpdateParamsPermission {
                security_address: false,
                limit: true,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that if you have permissions, you can change all parameters of the tokenfactory
/// module (new style parameter changes). NOTE: this does not check that the
/// parameters have actually been changed.
#[test]
pub fn test_execute_execute_message_update_params_tokenfactory_authorized() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/osmosis.tokenfactory.v1beta1.MsgUpdateParams",
            "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
            "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateTokenfactoryParamsPermission(
            TokenfactoryUpdateParamsPermission {
                denom_creation_fee: true,
                denom_creation_gas_consume: true,
                fee_collector_address: true,
                whitelisted_hooks: true,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap();
}
/// Checks that you can't change the denom_creation_fee if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_tokenfactory_unauthorized_denom_creation_fee() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/osmosis.tokenfactory.v1beta1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateTokenfactoryParamsPermission(
            TokenfactoryUpdateParamsPermission {
                denom_creation_fee: false,
                denom_creation_gas_consume: true,
                fee_collector_address: true,
                whitelisted_hooks: true,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {})
}

/// Checks that you can't change the denom_creation_gas_consume if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_tokenfactory_unauthorized_denom_creation_gas_consume(
) {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/osmosis.tokenfactory.v1beta1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateTokenfactoryParamsPermission(
            TokenfactoryUpdateParamsPermission {
                denom_creation_fee: true,
                denom_creation_gas_consume: false,
                fee_collector_address: true,
                whitelisted_hooks: true,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change the fee_collector_address if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_tokenfactory_unauthorized_fee_collector_address()
{
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/osmosis.tokenfactory.v1beta1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateTokenfactoryParamsPermission(
            TokenfactoryUpdateParamsPermission {
                denom_creation_fee: true,
                denom_creation_gas_consume: true,
                fee_collector_address: false,
                whitelisted_hooks: true,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change the whitelisted_hooks if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_tokenfactory_unauthorized_whitelisted_hooks() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/osmosis.tokenfactory.v1beta1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateTokenfactoryParamsPermission(
            TokenfactoryUpdateParamsPermission {
                denom_creation_fee: true,
                denom_creation_gas_consume: true,
                fee_collector_address: true,
                whitelisted_hooks: false,
            },
        )]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that if you have permissions, you can change all parameters of the dex
/// module (new style parameter changes). NOTE: this does not check that the
/// parameters have actually been changed.
#[test]
pub fn test_execute_execute_message_update_params_dex_authorized() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.dex.MsgUpdateParams",
            "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
            "params": {"fee_tiers":["1","2"],"paused":true,"max_jits_per_block":"25","good_til_purge_allowance":"540000"}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateDexParamsPermission(DexUpdateParamsPermission {
            fee_tiers: true,
            paused: true,
            max_jits_per_block: true,
            good_til_purge_allowance: true,
        })]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap();
}

/// Checks that you can't change the `fee_tiers` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_dex_unauthorized_fee_tiers() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.dex.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"fee_tiers":["1","2"],"paused":true,"max_jits_per_block":"25","good_til_purge_allowance":"540000"}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateDexParamsPermission(DexUpdateParamsPermission {
            fee_tiers: false,
            paused: true,
            max_jits_per_block: true,
            good_til_purge_allowance: true,
        })]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {})
}

/// Checks that you can't change `paused` if you don't have the permission to do so
/// (new style parameter changes).

#[test]
pub fn test_execute_execute_message_update_params_dex_unauthorized_paused() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.dex.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"fee_tiers":["1","2"],"paused":true,"max_jits_per_block":"25","good_til_purge_allowance":"540000"}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateDexParamsPermission(DexUpdateParamsPermission {
            fee_tiers: true,
            paused: false,
            max_jits_per_block: true,
            good_til_purge_allowance: true,
        })]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `max_jits_per_block` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_dex_unauthorized_max_jits_per_block() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.dex.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"fee_tiers":["1","2"],"paused":true,"max_jits_per_block":"25","good_til_purge_allowance":"540000"}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateDexParamsPermission(DexUpdateParamsPermission {
            fee_tiers: true,
            paused: true,
            max_jits_per_block: false,
            good_til_purge_allowance: true,
        })]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}
/// Checks that you can't change `good_til_purge_allowance` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_dex_unauthorized_good_til_purge_allowance() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.dex.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"fee_tiers":["1","2"],"paused":true,"max_jits_per_block":"25","good_til_purge_allowance":"540000"}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateDexParamsPermission(DexUpdateParamsPermission {
            fee_tiers: true,
            paused: true,
            max_jits_per_block: true,
            good_til_purge_allowance: false,
        })]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that if you have permissions, you can change all parameters of the dynamicfees
/// module (new style parameter changes). NOTE: this does not check that the
/// parameters have actually been changed.
#[test]
pub fn test_execute_execute_message_update_params_dynamicfees_authorized() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.dynamicfees.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"ntrn_prices":[{"denom":"utia","amount":"1.5"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let err = execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg.clone()])
        .unwrap_err();
    assert_eq!(err, Unauthorized {});

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateDynamicfeesParamsPermission(
            DynamicFeesUpdateParamsPermission { ntrn_prices: true },
        )]),
    )
    .unwrap();

    execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap();
}

/// Checks that you can't change `ntrn_prices` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_dynamicfees_unauthorized() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.dynamicfees.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"ntrn_prices":[{"denom":"utia","amount":"1.5"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let err = execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg.clone()])
        .unwrap_err();
    assert_eq!(err, Unauthorized {});

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateDynamicfeesParamsPermission(
            DynamicFeesUpdateParamsPermission { ntrn_prices: false },
        )]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that if you have permissions, you can change all parameters of the globalfee
/// module (new style parameter changes). NOTE: this does not check that the
/// parameters have actually been changed.
#[test]
pub fn test_execute_execute_message_update_params_globalfee_authorized() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/gaia.globalfee.v1beta1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"minimum_gas_prices":[{"denom":"utia","amount":"2.5"}],"bypass_min_fee_msg_types":["allowedMsgType2"],"max_total_bypass_min_fee_msg_gas_usage":100000}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let err = execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg.clone()])
        .unwrap_err();
    assert_eq!(err, Unauthorized {});

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateGlobalfeeParamsPermission(
            GlobalfeeUpdateParamsPermission {
                minimum_gas_prices: true,
                bypass_min_fee_msg_types: true,
                max_total_bypass_min_fee_msg_gas_usage: true,
            },
        )]),
    )
    .unwrap();

    execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap();
}

/// Checks that you can't change `minimum_gas_prices` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_globalfee_minimum_gas_prices_unauthorized() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/gaia.globalfee.v1beta1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"minimum_gas_prices":[{"denom":"utia","amount":"2.5"}],"bypass_min_fee_msg_types":["allowedMsgType2"],"max_total_bypass_min_fee_msg_gas_usage":100000}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let err = execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg.clone()])
        .unwrap_err();
    assert_eq!(err, Unauthorized {});

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateGlobalfeeParamsPermission(
            GlobalfeeUpdateParamsPermission {
                minimum_gas_prices: false,
                bypass_min_fee_msg_types: true,
                max_total_bypass_min_fee_msg_gas_usage: true,
            },
        )]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `bypass_min_fee_msg_types` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_globalfee_bypass_min_fee_msg_types_unauthorized()
{
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/gaia.globalfee.v1beta1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"minimum_gas_prices":[{"denom":"utia","amount":"2.5"}],"bypass_min_fee_msg_types":["allowedMsgType2"],"max_total_bypass_min_fee_msg_gas_usage":100000}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let err = execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg.clone()])
        .unwrap_err();
    assert_eq!(err, Unauthorized {});

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateGlobalfeeParamsPermission(
            GlobalfeeUpdateParamsPermission {
                minimum_gas_prices: true,
                bypass_min_fee_msg_types: false,
                max_total_bypass_min_fee_msg_gas_usage: true,
            },
        )]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `max_total_bypass_min_fee_msg_gas_usage` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_globalfee_max_total_bypass_min_fee_msg_gas_usage_unauthorized(
) {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/gaia.globalfee.v1beta1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"minimum_gas_prices":[{"denom":"utia","amount":"2.5"}],"bypass_min_fee_msg_types":["allowedMsgType2"],"max_total_bypass_min_fee_msg_gas_usage":100000}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let err = execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg.clone()])
        .unwrap_err();
    assert_eq!(err, Unauthorized {});

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateGlobalfeeParamsPermission(
            GlobalfeeUpdateParamsPermission {
                minimum_gas_prices: true,
                bypass_min_fee_msg_types: true,
                max_total_bypass_min_fee_msg_gas_usage: false,
            },
        )]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that if you have permissions, you can change all parameters of the consumer
/// module (new style parameter changes). NOTE: this does not check that the
/// parameters have actually been changed.
#[test]
pub fn test_execute_execute_message_update_params_consumer_authorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let err = execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg.clone()])
        .unwrap_err();
    assert_eq!(err, Unauthorized {});

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap();
}

/// Checks that you can't change `enabled`. It is not allowed to change `enabled` param.
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_disable_unauthorized() {
    let mut params = default_consumer_params();
    params.enabled = false;
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `blocks_per_distribution_transmission` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_blocks_per_distribution_transmission_unauthorized(
) {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: false,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `distribution_transmission_channel` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_distribution_transmission_channel_unauthorized(
) {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: false,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `provider_fee_pool_addr_str` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_provider_fee_pool_addr_str_unauthorized()
{
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: false,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `ccv_timeout_period` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_ccv_timeout_period_unauthorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: false,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `transfer_timeout_period` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_transfer_timeout_period_unauthorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: false,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `consumer_redistribution_fraction` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_consumer_redistribution_fraction_unauthorized(
) {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: false,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `historical_entries` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_historical_entries_unauthorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: false,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `unbonding_period` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_unbonding_period_unauthorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: false,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `soft_opt_out_threshold` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_soft_opt_out_threshold_unauthorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: false,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `reward_denoms` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_reward_denoms_unauthorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: false,
            provider_reward_denoms: true,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `provider_reward_denoms` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_provider_reward_denoms_unauthorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: false,
            retry_delay_period: true,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change `retry_delay_period` if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_consumer_retry_delay_period_unauthorized() {
    let params = consumer_params_to_update();
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: format!(
                r#"{{"@type":"/interchain_security.ccv.consumer.v1.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {}}}"#,
                serde_json_wasm::to_string(&params).unwrap()
            )
            .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    let non_priv_info = message_info(&Addr::unchecked("addr1"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![UpdateCCVParamsPermission(CCVUpdateParamsPermission {
            blocks_per_distribution_transmission: true,
            distribution_transmission_channel: true,
            provider_fee_pool_addr_str: true,
            ccv_timeout_period: true,
            transfer_timeout_period: true,
            consumer_redistribution_fraction: true,
            historical_entries: true,
            unbonding_period: true,
            soft_opt_out_threshold: true,
            reward_denoms: true,
            provider_reward_denoms: true,
            retry_delay_period: false,
        })]),
    )
    .unwrap();

    let err =
        execute_execute_messages(deps.as_mut(), non_priv_info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can update a legacy param if you have the necessary ALLOW_ONLY permission.
#[test]
pub fn test_execute_execute_message_param_change_success() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ParamChangeProposal(ParamChangeProposal {
            title: "test_proposal".to_string(),
            description: "Test proposal".to_string(),
            param_changes: vec![ParamChange {
                subspace: "globalfee".to_string(),
                key: "MinimumGasPricesParam".to_string(),
                value: "1000".to_string(),
            }],
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![ParamChangePermission(ParamChangePermissionType {
            params: vec![ParamPermission {
                subspace: "globalfee".to_string(),
                key: "MinimumGasPricesParam".to_string(),
            }],
        })]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap();
}

/// Checks that you can not update a legacy param without the necessary ALLOW_ONLY permission.
/// In this scenario, the subspace permission is correct, but the key permission is incorrect.
#[test]
pub fn test_execute_execute_message_param_change_unauthorized_key() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ParamChangeProposal(ParamChangeProposal {
            title: "test_proposal".to_string(),
            description: "Test proposal".to_string(),
            param_changes: vec![ParamChange {
                subspace: "globalfee".to_string(),
                key: "MinimumGasPricesParam".to_string(),
                value: "1000".to_string(),
            }],
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![ParamChangePermission(ParamChangePermissionType {
            params: vec![ParamPermission {
                subspace: "globalfee".to_string(),
                key: "0xdeadbeef".to_string(),
            }],
        })]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can not update a legacy param without the necessary ALLOW_ONLY permission.
/// In this scenario, the key permission is correct, but the subspace permission is incorrect.
#[test]
pub fn test_execute_execute_message_param_change_unauthorized_subspace() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ParamChangeProposal(ParamChangeProposal {
            title: "test_proposal".to_string(),
            description: "Test proposal".to_string(),
            param_changes: vec![ParamChange {
                subspace: "globalfee".to_string(),
                key: "MinimumGasPricesParam".to_string(),
                value: "1000".to_string(),
            }],
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![ParamChangePermission(ParamChangePermissionType {
            params: vec![ParamPermission {
                subspace: "0xdeadbeef".to_string(),
                key: "MinimumGasPricesParam".to_string(),
            }],
        })]),
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can not execute a message that the contract manager
/// doesn't have specific handlers for.
#[test]
pub fn test_execute_execute_unknown_message() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = message_info(&Addr::unchecked("neutron_dao_address"), &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowOnly(vec![ParamChangePermission(ParamChangePermissionType {
            params: vec![ParamPermission {
                subspace: "0xdeadbeef".to_string(),
                key: "0xdeadbeef".to_string(),
            }],
        })]),
    )
    .unwrap();

    let msg = CosmosMsg::Bank(BankMsg::Burn {
        amount: vec![Coin::new(Uint128::new(42u128), "0xdeadbeef".to_string())],
    });

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});

    let msg = CosmosMsg::Custom(NeutronMsg::BurnTokens {
        denom: "0xdeadbeef".to_string(),
        amount: Uint128::new(42),
        burn_from_address: "".to_string(),
    });

    let info = message_info(&Addr::unchecked("addr1"), &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}
