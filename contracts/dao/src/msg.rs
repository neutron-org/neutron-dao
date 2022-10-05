use neutron_bindings::msg::ParamChange;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// The contract's owner
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership {
        new_owner: String,
    },
    AddAdmin {
        new_admin: String,
    },
    SubmitTextProposal {
        title: String,
        description: String,
    },
    SubmitChangeParamsProposal {
        title: String,
        description: String,
        params_change: Vec<ParamChange>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    Config {},
}

pub type ConfigResponse = InstantiateMsg;
