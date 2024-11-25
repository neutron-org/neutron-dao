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
                match permissions.get(&PermissionType::UpdateCronParamsPermission) {
                    Some(Permission::UpdateCronParamsPermission(cron_update_params)) => {
                        Some(cron_update_params.clone())
                    }
                    _ => None,
                }
            }
        }
    }

    pub fn get_tokenfactory_update_param_permission(
        &self,
    ) -> Option<TokenfactoryUpdateParamsPermission> {
        match self {
            Strategy::AllowAll => Some(TokenfactoryUpdateParamsPermission {
                denom_creation_fee: true,
                denom_creation_gas_consume: true,
                fee_collector_address: true,
                whitelisted_hooks: true,
            }),
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::UpdateTokenfactoryParamsPermission) {
                    Some(Permission::UpdateTokenfactoryParamsPermission(
                        tokenfactory_update_params,
                    )) => Some(tokenfactory_update_params.clone()),
                    _ => None,
                }
            }
        }
    }

    pub fn get_dex_update_param_permission(&self) -> Option<DexUpdateParamsPermission> {
        match self {
            Strategy::AllowAll => Some(DexUpdateParamsPermission {
                fee_tiers: true,
                paused: true,
                max_jits_per_block: true,
                good_til_purge_allowance: true,
            }),
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::UpdateDexParamsPermission) {
                    Some(Permission::UpdateDexParamsPermission(dex_update_params)) => {
                        Some(dex_update_params.clone())
                    }
                    _ => None,
                }
            }
        }
    }

    pub fn get_dynamicfees_update_param_permission(
        &self,
    ) -> Option<DynamicFeesUpdateParamsPermission> {
        match self {
            Strategy::AllowAll => Some(DynamicFeesUpdateParamsPermission { ntrn_prices: true }),
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::UpdateDynamicfeesParamsPermission) {
                    Some(Permission::UpdateDynamicfeesParamsPermission(
                        dynamicfees_update_params,
                    )) => Some(dynamicfees_update_params.clone()),
                    _ => None,
                }
            }
        }
    }

    pub fn get_globalfee_update_param_permission(&self) -> Option<GlobalfeeUpdateParamsPermission> {
        match self {
            Strategy::AllowAll => Some(GlobalfeeUpdateParamsPermission {
                minimum_gas_prices: true,
                bypass_min_fee_msg_types: true,
                max_total_bypass_min_fee_msg_gas_usage: true,
            }),
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::UpdateGlobalfeeParamsPermission) {
                    Some(Permission::UpdateGlobalfeeParamsPermission(globalfee_update_params)) => {
                        Some(globalfee_update_params.clone())
                    }
                    _ => None,
                }
            }
        }
    }

    pub fn get_ccv_update_param_permission(&self) -> Option<CCVUpdateParamsPermission> {
        match self {
            Strategy::AllowAll => Some(CCVUpdateParamsPermission {
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
            }),
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::UpdateCCVParamsPermission) {
                    Some(Permission::UpdateCCVParamsPermission(ccv_update_params)) => {
                        Some(ccv_update_params.clone())
                    }
                    _ => None,
                }
            }
        }
    }

    pub fn has_software_upgrade_permission(&self) -> bool {
        match self {
            Strategy::AllowAll => true,
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::SoftwareUpgradePermission) {
                    Some(Permission::SoftwareUpgradePermission(software_upgrade_params)) => {
                        software_upgrade_params.upgrade
                    }
                    _ => false,
                }
            }
        }
    }

    pub fn has_cancel_software_upgrade_permission(&self) -> bool {
        match self {
            Strategy::AllowAll => true,
            Strategy::AllowOnly(permissions) => {
                match permissions.get(&PermissionType::SoftwareUpgradePermission) {
                    Some(Permission::SoftwareUpgradePermission(software_upgrade_params)) => {
                        software_upgrade_params.cancel_upgrade
                    }
                    _ => false,
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
    UpdateCronParamsPermission(CronUpdateParamsPermission),
    UpdateTokenfactoryParamsPermission(TokenfactoryUpdateParamsPermission),
    UpdateDexParamsPermission(DexUpdateParamsPermission),
    UpdateDynamicfeesParamsPermission(DynamicFeesUpdateParamsPermission),
    UpdateGlobalfeeParamsPermission(GlobalfeeUpdateParamsPermission),
    #[serde(rename = "update_ccv_params_permission")]
    UpdateCCVParamsPermission(CCVUpdateParamsPermission),
    CronPermission(CronPermission),
    SoftwareUpgradePermission(SoftwareUpgradePermission),
}

impl From<Permission> for PermissionType {
    fn from(value: Permission) -> Self {
        match value {
            Permission::ParamChangePermission(_) => PermissionType::ParamChangePermission,
            Permission::UpdateCronParamsPermission(_) => PermissionType::UpdateCronParamsPermission,
            Permission::UpdateTokenfactoryParamsPermission(_) => {
                PermissionType::UpdateTokenfactoryParamsPermission
            }
            Permission::UpdateDexParamsPermission(_) => PermissionType::UpdateDexParamsPermission,
            Permission::CronPermission(_) => PermissionType::CronPermission,
            Permission::SoftwareUpgradePermission(_) => PermissionType::SoftwareUpgradePermission,
            Permission::UpdateDynamicfeesParamsPermission(_) => {
                PermissionType::UpdateDynamicfeesParamsPermission
            }
            Permission::UpdateGlobalfeeParamsPermission(_) => {
                PermissionType::UpdateGlobalfeeParamsPermission
            }
            Permission::UpdateCCVParamsPermission(_) => PermissionType::UpdateCCVParamsPermission,
        }
    }
}

#[cw_serde]
#[derive(Hash, Eq)]
pub enum PermissionType {
    ParamChangePermission,
    UpdateCronParamsPermission,
    UpdateTokenfactoryParamsPermission,
    UpdateDexParamsPermission,
    UpdateDynamicfeesParamsPermission,
    UpdateGlobalfeeParamsPermission,
    UpdateCCVParamsPermission,
    CronPermission,
    SoftwareUpgradePermission,
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
pub struct DexUpdateParamsPermission {
    pub fee_tiers: bool,
    pub paused: bool,
    pub max_jits_per_block: bool,
    pub good_til_purge_allowance: bool,
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct DynamicFeesUpdateParamsPermission {
    pub ntrn_prices: bool,
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct GlobalfeeUpdateParamsPermission {
    pub minimum_gas_prices: bool,
    pub bypass_min_fee_msg_types: bool,
    pub max_total_bypass_min_fee_msg_gas_usage: bool,
}

#[cw_serde]
#[derive(Eq)]
#[serde(rename_all = "snake_case")]
pub struct CCVUpdateParamsPermission {
    pub blocks_per_distribution_transmission: bool,
    pub distribution_transmission_channel: bool,
    pub provider_fee_pool_addr_str: bool,
    pub ccv_timeout_period: bool,
    pub transfer_timeout_period: bool,
    pub consumer_redistribution_fraction: bool,
    pub historical_entries: bool,
    pub unbonding_period: bool,
    // !!! DEPRECATED !!! soft_opt_out_threshold is deprecated.
    // see https://github.com/cosmos/interchain-security/blob/main/docs/docs/adrs/adr-015-partial-set-security.md
    pub soft_opt_out_threshold: bool,
    pub reward_denoms: bool,
    pub provider_reward_denoms: bool,
    pub retry_delay_period: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SoftwareUpgradePermission {
    pub upgrade: bool,
    pub cancel_upgrade: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalExecuteMessageJSON {
    #[serde(rename = "@type")]
    pub type_field: String,
}
