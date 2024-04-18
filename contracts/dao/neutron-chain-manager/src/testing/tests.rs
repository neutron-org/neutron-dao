use crate::contract::{
    execute_add_permissions, execute_execute_messages, execute_remove_strategy, instantiate,
};
use crate::error::ContractError::{InvalidDemotion, PermissionTypeNotFound, Unauthorized};
use crate::msg::{CronUpdateParamsPermission, InstantiateMsg};
use crate::msg::{ParamChangePermission, ParamPermission};
use crate::permission::Permission;
use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Uint128};
use neutron_sdk::bindings::msg::{
    AdminProposal, ClientUpdateProposal, NeutronMsg, ParamChange, ParamChangeProposal,
    ProposalExecuteMessage,
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    // Scenario 1: An ALLOW_ALL strategy is added for a new address (passes).
    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::AllowAll],
    )
    .unwrap();

    // Scenario 2: An ALLOW_ONLY strategy is added for a new address (passes).
    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr2".to_string()),
        vec![
            Permission::AddCronPermission,
            Permission::RemoveCronPermission,
        ],
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr2".to_string()),
        vec![
            Permission::AddCronPermission,
            Permission::RemoveCronPermission,
        ],
    )
    .unwrap();

    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr2".to_string()),
        vec![Permission::AllowAll],
    )
    .unwrap();

    let info = mock_info("addr2", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr3".to_string()),
        vec![Permission::AllowAll],
    )
    .unwrap();
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::AllowAll],
    )
    .unwrap();
    execute_remove_strategy(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
    )
    .unwrap();

    let info = mock_info("addr1", &[]);
    let err = execute_add_permissions(
        deps.as_mut(),
        info,
        Addr::unchecked("addr1".to_string()),
        vec![Permission::AllowAll],
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::CronUpdateParamsPermission(
            CronUpdateParamsPermission {
                security_address: true,
                limit: true,
            },
        )],
    )
    .unwrap();

    let info = mock_info("addr1", &[]);
    execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap();
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::CronUpdateParamsPermission(
            CronUpdateParamsPermission {
                security_address: true,
                limit: false,
            },
        )],
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::CronUpdateParamsPermission(
            CronUpdateParamsPermission {
                security_address: false,
                limit: true,
            },
        )],
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::ParamChangePermission(ParamChangePermission {
            params: vec![ParamPermission {
                subspace: "globalfee".to_string(),
                key: "MinimumGasPricesParam".to_string(),
            }],
        })],
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::ParamChangePermission(ParamChangePermission {
            params: vec![ParamPermission {
                subspace: "globalfee".to_string(),
                key: "0xdeadbeef".to_string(),
            }],
        })],
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::ParamChangePermission(ParamChangePermission {
            params: vec![ParamPermission {
                subspace: "0xdeadbeef".to_string(),
                key: "MinimumGasPricesParam".to_string(),
            }],
        })],
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
            initial_address: Addr::unchecked("neutron_dao_address".to_string()),
        },
    )
    .unwrap();

    let info = mock_info("neutron_dao_address", &[]);
    execute_add_permissions(
        deps.as_mut(),
        info.clone(),
        Addr::unchecked("addr1".to_string()),
        vec![Permission::ParamChangePermission(ParamChangePermission {
            params: vec![ParamPermission {
                subspace: "0xdedbeef".to_string(),
                key: "0xdedbeef".to_string(),
            }],
        })],
    )
    .unwrap();

    let msg = CosmosMsg::Bank(BankMsg::Burn {
        amount: vec![Coin::new(42, "0xdeadbeef".to_string())],
    });

    let info = mock_info("addr1", &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(
        err,
        PermissionTypeNotFound("no registered persimmisions for the message".to_string())
    );

    let msg = CosmosMsg::Custom(NeutronMsg::BurnTokens {
        denom: "0xdeadbeef".to_string(),
        amount: Uint128::new(42),
        burn_from_address: "".to_string(),
    });

    let info = mock_info("addr1", &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(
        err,
        PermissionTypeNotFound("no registered persimmisions for the message".to_string())
    );

    let msg = CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal {
        admin_proposal: AdminProposal::ClientUpdateProposal(ClientUpdateProposal {
            title: "0xdeadbeef".to_string(),
            description: "0xdeadbeef".to_string(),
            subject_client_id: "0xdeadbeef".to_string(),
            substitute_client_id: "0xdeadbeef".to_string(),
        }),
    });

    let info = mock_info("addr1", &[]);
    let err = execute_execute_messages(deps.as_mut(), info.clone(), vec![msg]).unwrap_err();
    assert_eq!(
        err,
        PermissionTypeNotFound(
            "no registered admin proposal persimmisions for the message".to_string()
        )
    );
}
