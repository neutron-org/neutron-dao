use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg};
use neutron_sdk::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::permission::Permission;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    /// Defines the address for the initial setup.
    pub initial_address: Addr,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ParamChangePermission {
    pub params: Vec<ParamPermission>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Hash)]
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
