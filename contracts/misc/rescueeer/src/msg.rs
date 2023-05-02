use cosmwasm_std::CosmosMsg;
use neutron_bindings::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    pub owner: String,
    pub true_admin: String,
    pub eol: u64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Execute {
        msgs: Vec<CosmosMsg<NeutronMsg>>,
    },
    TransferAdmin {
        address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
