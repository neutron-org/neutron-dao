use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg};
use neutron_sdk::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::permission::Permission;

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Defines the address for the initial setup.
    pub initial_address: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddPermissions {
        address: Addr,
        permissions: Vec<Permission>,
    },
    RemovePermissions {
        address: Addr,
    },
    ExecuteMessages {
        messages: Vec<CosmosMsg<NeutronMsg>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec < Permission >)]
    Permissions {},
}
#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct ParamChangePermission {
    pub params: Vec<ParamPermission>,
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
#[derive(Eq, Hash)]
pub struct ParamPermission {
    pub subspace: String,
    pub key: String,
}

#[cw_serde]
pub enum UpdateParamsPermission {
    CronUpdateParamsPermission(CronUpdateParamsPermission),
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct CronUpdateParamsPermission {
    pub security_address: bool,
    pub limit: bool,
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
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
