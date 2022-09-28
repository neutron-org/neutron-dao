use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use neutron_bindings::msg::ParamChange;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// The contract's owner
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),
    AddAdmin (String),
    SubmitTextProposal(String, String),
    SubmitChangeParamProposal(String, String, Vec<ParamChange>),
    SubmitCommunityPoolSpendProposal(String, String, String),
    SubmitClientUpdateProposal(String, String, String, String),
    SubmitSoftwareUpdateProposal(String, String),
    SubmitCancelSoftwareUpdateProposal(String, String)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    Config {},
}

pub type ConfigResponse = InstantiateMsg;

/// MsgTextPoposal defines a SDK message for submission of text proposal
#[derive(Clone, PartialEq)]
pub struct MsgTextProposal {
    pub title: String,
    pub description: String,
}
