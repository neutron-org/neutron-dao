use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg};
use neutron_sdk::bindings::msg::{NeutronMsg, ParamChange};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    /// Defines the address for the initial strategy.
    pub initial_strategy_address: Addr,
    /// Defines the initial strategy. Must be an ALLOW_ALL strategy.
    pub initial_strategy: Strategy,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddStrategy {
        address: Addr,
        strategy: Strategy,
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
    #[returns(Vec < Strategy >)]
    Strategies {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum Strategy {
    AllowAll,
    AllowOnly(Vec<Permission>),
}

impl Strategy {
    pub fn has_cron_add_schedule_permission(&self) -> bool {
        match self {
            Strategy::AllowAll => true,
            Strategy::AllowOnly(permissions) => {
                let mut has_permission = false;
                for permission in permissions {
                    if let Permission::CronPermission(cron_permission) = permission {
                        has_permission = cron_permission.add_schedule
                    }
                }

                has_permission
            }
        }
    }
    pub fn has_cron_remove_schedule_permission(&self) -> bool {
        match self {
            Strategy::AllowAll => true,
            Strategy::AllowOnly(permissions) => {
                let mut has_permission = false;
                for permission in permissions {
                    if let Permission::CronPermission(cron_permission) = permission {
                        has_permission = cron_permission.remove_schedule
                    }
                }

                has_permission
            }
        }
    }
    pub fn has_param_change_permission(&self, param_change: ParamChange) -> bool {
        match self {
            Strategy::AllowAll => true,
            Strategy::AllowOnly(permissions) => {
                for permission in permissions {
                    if let Permission::ParamChangePermission(param_change_permissions) = permission
                    {
                        for param_change_permission in param_change_permissions.params.clone() {
                            if param_change.subspace == param_change_permission.subspace
                                && param_change.key == param_change_permission.key
                            {
                                return true;
                            }
                        }
                    }
                }

                false
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
                for permission in permissions {
                    if let Permission::UpdateParamsPermission(update_params_permission) = permission
                    {
                        return match update_params_permission {
                            UpdateParamsPermission::CronUpdateParamsPermission(
                                cron_update_params,
                            ) => Some(cron_update_params.clone()),
                        };
                    }
                }

                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum Permission {
    // Deprecated, for legacy parameter updates using `params` module.
    ParamChangePermission(ParamChangePermission),
    // For new-style parameter updates.
    UpdateParamsPermission(UpdateParamsPermission),
    CronPermission(CronPermission),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ParamChangePermission {
    pub params: Vec<ParamPermission>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ParamPermission {
    pub subspace: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum UpdateParamsPermission {
    CronUpdateParamsPermission(CronUpdateParamsPermission),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CronUpdateParamsPermission {
    pub security_address: bool,
    pub limit: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CronPermission {
    pub add_schedule: bool,
    pub remove_schedule: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalExecuteMessageJSON {
    #[serde(rename = "@type")]
    pub type_field: String,
}
