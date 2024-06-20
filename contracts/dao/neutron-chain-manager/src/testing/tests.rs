use crate::contract::{
    execute_add_strategy, execute_execute_messages, execute_remove_strategy, instantiate,
};
use crate::error::ContractError::{InvalidDemotion, Unauthorized};
use crate::msg::InstantiateMsg;
use crate::msg::Permission::{
    CronPermission, ParamChangePermission, UpdateCronParamsPermission,
    UpdateTokenfactoryParamsPermission,
};
use crate::msg::{
    CronPermission as CronPermissionType, CronUpdateParamsPermission, StrategyMsg,
    TokenfactoryUpdateParamsPermission,
};
use crate::msg::{ParamChangePermission as ParamChangePermissionType, ParamPermission};
use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Uint128};
use neutron_sdk::bindings::msg::{
    AdminProposal, NeutronMsg, ParamChange, ParamChangeProposal, ProposalExecuteMessage,
};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("addr1", &[]);

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
    let info = mock_info("addr1", &[]);

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
    let info = mock_info("neutron_dao_address", &[]);
    execute_add_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        StrategyMsg::AllowAll,
    )
    .unwrap();

    // Scenario 2: An ALLOW_ONLY strategy is added for a new address (passes).
    let info = mock_info("neutron_dao_address", &[]);
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
    let info = mock_info("addr1", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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
    let info = mock_info("addr2", &[]);
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
    let info = mock_info("addr1", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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
    let info = mock_info("addr1", &[]);
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
    let info = mock_info("addr1", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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
    let info = mock_info("addr1", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
    let info = mock_info("neutron_dao_address", &[]);

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
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
            message: r#"{"@type":"/neutron.tokenfactory.MsgUpdateParams",
            "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
            "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
    execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap();
}
/// Checks that you can't change the denom_creation_fee if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_tokenfactory_unauthorized_denom_creation_fee() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.tokenfactory.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
            message: r#"{"@type":"/neutron.tokenfactory.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
            message: r#"{"@type":"/neutron.tokenfactory.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can't change the whitelisted_hooks if you don't have the permission to do so
/// (new style parameter changes).
#[test]
pub fn test_execute_execute_message_update_params_tokenfactory_unauthorized_whitelisted_hooks() {
    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage {
            message: r#"{"@type":"/neutron.tokenfactory.MsgUpdateParams",
             "authority":"neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z",
             "params": {"denom_creation_fee": [{"denom": "untrn", "amount": "100"}], "denom_creation_gas_consume": "100", "fee_collector_address": "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z", "whitelisted_hooks": [{"code_id": "1", "denom_creator": "neutron1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts8g30fq"}]}}"#
                .to_string(),
        }),
    });

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
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
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
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
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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

    let info = mock_info("addr1", &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}

/// Checks that you can not execute a message that the contract manager
/// doesn't have specific handlers for.
#[test]
pub fn test_execute_execute_unknown_message() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("neutron_dao_address", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            initial_strategy_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
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
        amount: vec![Coin::new(42, "0xdeadbeef".to_string())],
    });

    let info = mock_info("addr1", &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});

    let msg = CosmosMsg::Custom(NeutronMsg::BurnTokens {
        denom: "0xdeadbeef".to_string(),
        amount: Uint128::new(42),
        burn_from_address: "".to_string(),
    });

    let info = mock_info("addr1", &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(err, Unauthorized {});
}
