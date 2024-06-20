use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg};
use neutron_sdk::bindings::msg::{NeutronMsg, ParamChange};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{json::JsonString, serde_as};

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Defines the address for the initial strategy.
    pub initial_strategy_address: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddStrategy {
        address: Addr,
        strategy: StrategyMsg,
    },
    RemoveStrategy {
        address: Addr,
    },
    ExecuteMessages {
        messages: Vec<CosmosMsg<NeutronMsg>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec < StrategyMsg >)]
    Strategies {},
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

// StrategyMsg is used only as UI struct to simplify intaraction with the contract
// Internally we work with `Strategy`
#[cw_serde]
pub enum StrategyMsg {
    AllowAll,
    AllowOnly(Vec<Permission>),
}

#[derive(JsonSchema)]
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Strategy {
    AllowAll,
    // the macro param required because serde allows only string as keys of hashmap during serialisation
    AllowOnly(#[serde_as(as = "HashMap<JsonString, _>")] HashMap<PermissionType, Permission>),
}

impl From<StrategyMsg> for Strategy {
    fn from(value: StrategyMsg) -> Self {
        match value {
            StrategyMsg::AllowAll => Strategy::AllowAll,
            StrategyMsg::AllowOnly(list_permissions) => {
                let mut perms: HashMap<PermissionType, Permission> = HashMap::new();
                for p in list_permissions {
                    perms.insert(p.clone().into(), p);
                }
                Strategy::AllowOnly(perms)
            }
        }
    }
}

impl Strategy {
    pub fn has_cron_add_schedule_permission(&self) -> bool {
        match self {
            Strategy::AllowAll => true,
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::CronPermission) {
                    Some(Permission::CronPermission(permission)) => permission.add_schedule,
                    _ => false,
                }
            }
        }
    }
    pub fn has_cron_remove_schedule_permission(&self) -> bool {
        match self {
            Strategy::AllowAll => true,
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::CronPermission) {
                    Some(Permission::CronPermission(permission)) => permission.remove_schedule,
                    _ => false,
                }
            }
        }
    }
    pub fn has_param_change_permission(&self, param_change: ParamChange) -> bool {
        match self {
            Strategy::AllowAll => true,
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::ParamChangePermission) {
                    Some(Permission::ParamChangePermission(param_change_permissions)) => {
                        for param_change_permission in param_change_permissions.params.clone() {
                            if param_change.subspace == param_change_permission.subspace
                                && param_change.key == param_change_permission.key
                            {
                                return true;
                            }
                        }
                        false
                    }
                    _ => false,
                }
            }
        }
    }

    pub fn get_cron_update_param_permission(&self) -> Option<CronUpdateParamsPermission> {
        match self {
            Strategy::AllowAll => Some(CronUpdateParamsPermission {
                security_address: true,
                limit: true,
            }),
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::UpdateParamsPermission) {
                    Some(Permission::UpdateParamsPermission(
                        UpdateParamsPermission::CronUpdateParamsPermission(cron_update_params),
                    )) => Some(cron_update_params.clone()),
                    _ => None,
                }
            }
        }
    }

    pub fn get_tokenfactory_update_param_permission(&self) -> Option<TokenfactoryUpdateParamsPermission> {
        match self {
            Strategy::AllowAll => Some(TokenfactoryUpdateParamsPermission {
                denom_creation_fee: true,
                denom_creation_gas_consume: true,
                fee_collector_address: true,
                whitelisted_hooks: true,
            }),
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::UpdateParamsPermission) {
                    Some(Permission::UpdateParamsPermission(
                        UpdateParamsPermission::TokenfactoryUpdateParamsPermission(tokenfactory_update_params),
                    )) => Some(tokenfactory_update_params.clone()),
                    _ => None,
                }
            }
        }
    }

}



#[cw_serde]
#[derive(Eq)]
pub enum Permission {
    // Deprecated, for legacy parameter updates using `params` module.
    ParamChangePermission(ParamChangePermission),
    // For new-style parameter updates.
    UpdateParamsPermission(UpdateParamsPermission),
    CronPermission(CronPermission),
    TokenfactoryPermission(TokenfactoryPermission),
}

impl From<Permission> for PermissionType {
    fn from(value: Permission) -> Self {
        match value {
            Permission::ParamChangePermission(_) => PermissionType::ParamChangePermission,
            Permission::UpdateParamsPermission(_) => PermissionType::UpdateParamsPermission,
            Permission::CronPermission(_) => PermissionType::CronPermission,
            Permission::TokenfactoryPermission(_) => PermissionType::TokenfactoryPermission,
        }
    }
}

#[cw_serde]
#[derive(Hash, Eq)]
pub enum PermissionType {
    ParamChangePermission,
    UpdateParamsPermission,
    CronPermission,
    TokenfactoryPermission,
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct ParamChangePermission {
    pub params: Vec<ParamPermission>,
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct ParamPermission {
    pub subspace: String,
    pub key: String,
}

#[cw_serde]
#[derive(Eq)]
pub enum UpdateParamsPermission {
    CronUpdateParamsPermission(CronUpdateParamsPermission),
    TokenfactoryUpdateParamsPermission(TokenfactoryUpdateParamsPermission),
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct CronUpdateParamsPermission {
    pub security_address: bool,
    pub limit: bool,
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct CronPermission {
    pub add_schedule: bool,
    pub remove_schedule: bool,
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct TokenfactoryUpdateParamsPermission {
    pub denom_creation_fee: bool,
    pub denom_creation_gas_consume: bool,
    pub fee_collector_address: bool,
    pub whitelisted_hooks: bool,
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct TokenfactoryPermission {
    pub denom_creation_fee: bool,
    pub denom_creation_gas_consume: bool,
    pub fee_collector_address: bool,
    pub whitelisted_hooks: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalExecuteMessageJSON {
    #[serde(rename = "@type")]
    pub type_field: String,
}
