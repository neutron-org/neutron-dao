use crate::contract::{execute, instantiate, migrate, query, CONTRACT_NAME, CONTRACT_VERSION};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, VotingVault};
use crate::state::{Config, VotingVaultState};
use crate::testing::mock_querier::{
    mock_dependencies, MOCK_VAULT_1, MOCK_VAULT_1_DESC, MOCK_VAULT_1_NAME, MOCK_VAULT_1_VP,
    MOCK_VAULT_2, MOCK_VAULT_2_DESC, MOCK_VAULT_2_NAME, MOCK_VAULT_2_VP, MOCK_VAULT_3,
    MOCK_VAULT_3_DESC, MOCK_VAULT_3_NAME, MOCK_VAULT_MEMBER,
};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_json, Addr, Deps, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_storage_plus::Item;
use cwd_interface::voting::{
    InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DAO_ADDR: &str = "dao";
const ADDR1: &str = "addr1";

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    assert_eq!(
        get_config(deps.as_ref(), env.clone()),
        Config {
            owner: Addr::unchecked(String::from(DAO_ADDR))
        }
    );

    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(env.block.height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Active,
        }]
    );
}

#[test]
fn test_instantiate_multiple_vaults() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![
                MOCK_VAULT_1.to_string(),
                MOCK_VAULT_2.to_string(),
                MOCK_VAULT_3.to_string(),
            ],
        },
    )
    .unwrap();

    assert_eq!(
        get_config(deps.as_ref(), env.clone()),
        Config {
            owner: Addr::unchecked(String::from(DAO_ADDR))
        }
    );

    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(env.block.height + 1)),
        vec![
            VotingVault {
                address: String::from(MOCK_VAULT_1),
                name: String::from(MOCK_VAULT_1_NAME),
                description: String::from(MOCK_VAULT_1_DESC),
                state: VotingVaultState::Active,
            },
            VotingVault {
                address: String::from(MOCK_VAULT_2),
                name: String::from(MOCK_VAULT_2_NAME),
                description: String::from(MOCK_VAULT_2_DESC),
                state: VotingVaultState::Active,
            },
            VotingVault {
                address: String::from(MOCK_VAULT_3),
                name: String::from(MOCK_VAULT_3_NAME),
                description: String::from(MOCK_VAULT_3_DESC),
                state: VotingVaultState::Active,
            }
        ]
    );
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADDR1, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    let err = update_config(deps.as_mut(), env, info, String::from(ADDR1)).unwrap_err();
    assert_eq!(err.to_string(), ContractError::Unauthorized {}.to_string());
}

#[test]
fn test_update_config_as_owner() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    update_config(deps.as_mut(), env.clone(), info, String::from(ADDR1)).unwrap();
    assert_eq!(
        get_config(deps.as_ref(), env),
        Config {
            owner: Addr::unchecked(ADDR1),
        }
    );
}

#[test]
fn test_query_dao() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    assert_eq!(get_dao(deps.as_ref(), env).as_str(), DAO_ADDR)
}

#[test]
fn test_query_info() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    assert_eq!(
        get_info(deps.as_ref(), env).info.contract.as_str(),
        "crates.io:neutron-voting-registry"
    )
}

#[test]
fn test_query_get_config() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    assert_eq!(
        get_config(deps.as_ref(), env),
        Config {
            owner: Addr::unchecked(DAO_ADDR),
        }
    )
}

#[test]
fn test_initial_voting_power() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let init_height = env.block.height;
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    // no voting power at height of initialisation
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(init_height)),
        vec![],
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(init_height)
        )
        .power,
        Uint128::zero(),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env.clone(), Some(init_height)).power,
        Uint128::zero(),
    );
    // some voting power at the next height after initialisation
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(init_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Active,
        }],
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(init_height + 1)
        )
        .power,
        Uint128::from(MOCK_VAULT_1_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env, Some(init_height + 1)).power,
        Uint128::from(MOCK_VAULT_1_VP),
    );
}

#[test]
fn test_add_vault() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    let add_new_vault_height = env.block.height + 50;
    env.block.height = add_new_vault_height;
    add_voting_vault(deps.as_mut(), env.clone(), info, String::from(MOCK_VAULT_2)).unwrap();

    // no voting power from the new vault at height when the vault has been added to the registry
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(add_new_vault_height)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Active,
        }]
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(add_new_vault_height)
        )
        .power,
        Uint128::from(MOCK_VAULT_1_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env.clone(), Some(add_new_vault_height)).power,
        Uint128::from(MOCK_VAULT_1_VP),
    );
    // additional voting power at the next height
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(add_new_vault_height + 1)),
        vec![
            VotingVault {
                address: String::from(MOCK_VAULT_1),
                name: String::from(MOCK_VAULT_1_NAME),
                description: String::from(MOCK_VAULT_1_DESC),
                state: VotingVaultState::Active,
            },
            VotingVault {
                address: String::from(MOCK_VAULT_2),
                name: String::from(MOCK_VAULT_2_NAME),
                description: String::from(MOCK_VAULT_2_DESC),
                state: VotingVaultState::Active,
            }
        ]
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(add_new_vault_height + 1)
        )
        .power,
        Uint128::from(MOCK_VAULT_1_VP + MOCK_VAULT_2_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env, Some(add_new_vault_height + 1)).power,
        Uint128::from(MOCK_VAULT_1_VP + MOCK_VAULT_2_VP),
    );
}

#[test]
fn test_add_vault_unauthorized() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info(ADDR1, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    let add_new_vault_height = env.block.height + 50;
    env.block.height = add_new_vault_height;
    let err =
        add_voting_vault(deps.as_mut(), env.clone(), info, String::from(MOCK_VAULT_2)).unwrap_err();
    assert_eq!(err.to_string(), ContractError::Unauthorized {}.to_string());

    assert_eq!(
        get_voting_vaults(deps.as_ref(), env, Some(add_new_vault_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Active,
        }]
    );
}

#[test]
fn test_add_already_existing_vault() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();

    let add_new_vault_height = env.block.height + 50;
    env.block.height = add_new_vault_height;
    let err =
        add_voting_vault(deps.as_mut(), env.clone(), info, String::from(MOCK_VAULT_1)).unwrap_err();
    assert_eq!(
        err.to_string(),
        ContractError::VotingVaultAlreadyExists {}.to_string()
    );

    assert_eq!(
        get_voting_vaults(deps.as_ref(), env, Some(add_new_vault_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Active,
        }]
    );
}

#[test]
fn test_vaults_activation_deactivation() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let init_height = env.block.height;
    let info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string(), MOCK_VAULT_2.to_string()],
        },
    )
    .unwrap();

    // some voting power at the next height after initialisation
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(init_height + 1)
        )
        .power,
        Uint128::from(MOCK_VAULT_1_VP + MOCK_VAULT_2_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env.clone(), Some(init_height + 1)).power,
        Uint128::from(MOCK_VAULT_1_VP + MOCK_VAULT_2_VP),
    );

    // deactivate the vault 1
    let deactivate_height = init_height + 50;
    env.block.height = deactivate_height;
    deactivate_voting_vault(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        MOCK_VAULT_1.to_string(),
    )
    .unwrap();

    // no voting power change at the height of deactivation
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(deactivate_height)),
        vec![
            VotingVault {
                address: String::from(MOCK_VAULT_1),
                name: String::from(MOCK_VAULT_1_NAME),
                description: String::from(MOCK_VAULT_1_DESC),
                state: VotingVaultState::Active,
            },
            VotingVault {
                address: String::from(MOCK_VAULT_2),
                name: String::from(MOCK_VAULT_2_NAME),
                description: String::from(MOCK_VAULT_2_DESC),
                state: VotingVaultState::Active,
            }
        ]
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(deactivate_height)
        )
        .power,
        Uint128::from(MOCK_VAULT_1_VP + MOCK_VAULT_2_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env.clone(), Some(deactivate_height)).power,
        Uint128::from(MOCK_VAULT_1_VP + MOCK_VAULT_2_VP),
    );
    // voting power is reduced at the next height
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(deactivate_height + 1)),
        vec![
            VotingVault {
                address: String::from(MOCK_VAULT_1),
                name: String::from(MOCK_VAULT_1_NAME),
                description: String::from(MOCK_VAULT_1_DESC),
                state: VotingVaultState::Inactive,
            },
            VotingVault {
                address: String::from(MOCK_VAULT_2),
                name: String::from(MOCK_VAULT_2_NAME),
                description: String::from(MOCK_VAULT_2_DESC),
                state: VotingVaultState::Active,
            }
        ]
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(deactivate_height + 1)
        )
        .power,
        Uint128::from(MOCK_VAULT_2_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env.clone(), Some(deactivate_height + 1)).power,
        Uint128::from(MOCK_VAULT_2_VP),
    );

    // activate the vault 1
    let activate_height = deactivate_height + 50;
    env.block.height = activate_height;
    activate_voting_vault(deps.as_mut(), env.clone(), info, MOCK_VAULT_1.to_string()).unwrap();

    // no voting power change at the height of activation
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(activate_height)),
        vec![
            VotingVault {
                address: String::from(MOCK_VAULT_1),
                name: String::from(MOCK_VAULT_1_NAME),
                description: String::from(MOCK_VAULT_1_DESC),
                state: VotingVaultState::Inactive,
            },
            VotingVault {
                address: String::from(MOCK_VAULT_2),
                name: String::from(MOCK_VAULT_2_NAME),
                description: String::from(MOCK_VAULT_2_DESC),
                state: VotingVaultState::Active,
            }
        ]
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(activate_height)
        )
        .power,
        Uint128::from(MOCK_VAULT_2_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env.clone(), Some(activate_height)).power,
        Uint128::from(MOCK_VAULT_2_VP),
    );
    // voting power is back at the next height
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(activate_height + 1)),
        vec![
            VotingVault {
                address: String::from(MOCK_VAULT_1),
                name: String::from(MOCK_VAULT_1_NAME),
                description: String::from(MOCK_VAULT_1_DESC),
                state: VotingVaultState::Active,
            },
            VotingVault {
                address: String::from(MOCK_VAULT_2),
                name: String::from(MOCK_VAULT_2_NAME),
                description: String::from(MOCK_VAULT_2_DESC),
                state: VotingVaultState::Active,
            }
        ]
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(activate_height + 1)
        )
        .power,
        Uint128::from(MOCK_VAULT_1_VP + MOCK_VAULT_2_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env.clone(), Some(activate_height + 1)).power,
        Uint128::from(MOCK_VAULT_1_VP + MOCK_VAULT_2_VP),
    );

    // historical query for past voting power when the vault 1 was inactive works the same way
    // (i.e. doesn't take the inactive vault into account) now when the vault is active
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(deactivate_height + 1)),
        vec![
            VotingVault {
                address: String::from(MOCK_VAULT_1),
                name: String::from(MOCK_VAULT_1_NAME),
                description: String::from(MOCK_VAULT_1_DESC),
                state: VotingVaultState::Inactive,
            },
            VotingVault {
                address: String::from(MOCK_VAULT_2),
                name: String::from(MOCK_VAULT_2_NAME),
                description: String::from(MOCK_VAULT_2_DESC),
                state: VotingVaultState::Active,
            }
        ]
    );
    assert_eq!(
        get_voting_power(
            deps.as_ref(),
            env.clone(),
            String::from(MOCK_VAULT_MEMBER),
            Some(deactivate_height + 1)
        )
        .power,
        Uint128::from(MOCK_VAULT_2_VP),
    );
    assert_eq!(
        get_total_voting_power(deps.as_ref(), env, Some(deactivate_height + 1)).power,
        Uint128::from(MOCK_VAULT_2_VP),
    );
}

#[test]
fn test_vaults_activation_deactivation_unauthorized() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let init_height = env.block.height;
    let owner_info = mock_info(DAO_ADDR, &[]);
    let stranger_info = mock_info(ADDR1, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        owner_info.clone(),
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(init_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Active,
        }],
    );

    // unauthorized deactivation
    let deactivate_height = init_height + 50;
    env.block.height = deactivate_height;
    let err = deactivate_voting_vault(
        deps.as_mut(),
        env.clone(),
        stranger_info.clone(),
        String::from(MOCK_VAULT_1),
    )
    .unwrap_err();
    assert_eq!(err.to_string(), ContractError::Unauthorized {}.to_string());
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(deactivate_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Active,
        }],
    );

    // authorized deactivation
    deactivate_voting_vault(
        deps.as_mut(),
        env.clone(),
        owner_info,
        String::from(MOCK_VAULT_1),
    )
    .unwrap();
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(deactivate_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Inactive,
        }],
    );

    // unauthorized activation
    let activate_height = deactivate_height + 50;
    env.block.height = activate_height;
    let err = activate_voting_vault(
        deps.as_mut(),
        env.clone(),
        stranger_info,
        String::from(MOCK_VAULT_1),
    )
    .unwrap_err();
    assert_eq!(err.to_string(), ContractError::Unauthorized {}.to_string());
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env, Some(activate_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Inactive,
        }],
    );
}

#[test]
fn test_vaults_activation_deactivation_wrong_switch() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let init_height = env.block.height;
    let owner_info = mock_info(DAO_ADDR, &[]);
    instantiate(
        deps.as_mut(),
        env.clone(),
        owner_info.clone(),
        InstantiateMsg {
            owner: String::from(DAO_ADDR),
            voting_vaults: vec![MOCK_VAULT_1.to_string()],
        },
    )
    .unwrap();
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(init_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Active,
        }],
    );

    // activation of an Active vault
    let err = activate_voting_vault(
        deps.as_mut(),
        env.clone(),
        owner_info.clone(),
        String::from(MOCK_VAULT_1),
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        ContractError::VotingVaultAlreadyActive {}.to_string()
    );

    // deactivation
    let deactivate_height = init_height + 50;
    env.block.height = deactivate_height;
    deactivate_voting_vault(
        deps.as_mut(),
        env.clone(),
        owner_info.clone(),
        String::from(MOCK_VAULT_1),
    )
    .unwrap();
    assert_eq!(
        get_voting_vaults(deps.as_ref(), env.clone(), Some(deactivate_height + 1)),
        vec![VotingVault {
            address: String::from(MOCK_VAULT_1),
            name: String::from(MOCK_VAULT_1_NAME),
            description: String::from(MOCK_VAULT_1_DESC),
            state: VotingVaultState::Inactive,
        }],
    );

    // deactivation of an Inactive vault
    let err = deactivate_voting_vault(deps.as_mut(), env, owner_info, String::from(MOCK_VAULT_1))
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        ContractError::VotingVaultAlreadyInactive {}.to_string()
    );
}

fn get_voting_vaults(deps: Deps, env: Env, height: Option<u64>) -> Vec<VotingVault> {
    let res = query(deps, env, QueryMsg::VotingVaults { height }).unwrap();
    from_json(res).unwrap()
}

fn get_config(deps: Deps, env: Env) -> Config {
    let res = query(deps, env, QueryMsg::Config {}).unwrap();
    from_json(res).unwrap()
}

fn get_dao(deps: Deps, env: Env) -> Addr {
    let res = query(deps, env, QueryMsg::Dao {}).unwrap();
    from_json(res).unwrap()
}

fn get_info(deps: Deps, env: Env) -> InfoResponse {
    let res = query(deps, env, QueryMsg::Info {}).unwrap();
    from_json(res).unwrap()
}

fn get_voting_power(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> VotingPowerAtHeightResponse {
    let res = query(deps, env, QueryMsg::VotingPowerAtHeight { address, height }).unwrap();
    from_json(res).unwrap()
}

fn get_total_voting_power(deps: Deps, env: Env, height: Option<u64>) -> TotalPowerAtHeightResponse {
    let res = query(deps, env, QueryMsg::TotalPowerAtHeight { height }).unwrap();
    from_json(res).unwrap()
}

fn add_voting_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault: String,
) -> Result<Response, ContractError> {
    execute(
        deps,
        env,
        info,
        ExecuteMsg::AddVotingVault {
            new_voting_vault_contract: vault,
        },
    )
}

fn activate_voting_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault: String,
) -> Result<Response, ContractError> {
    execute(
        deps,
        env,
        info,
        ExecuteMsg::ActivateVotingVault {
            voting_vault_contract: vault,
        },
    )
}

fn deactivate_voting_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault: String,
) -> Result<Response, ContractError> {
    execute(
        deps,
        env,
        info,
        ExecuteMsg::DeactivateVotingVault {
            voting_vault_contract: vault,
        },
    )
}

fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    execute(deps, env, info, ExecuteMsg::UpdateConfig { owner })
}
