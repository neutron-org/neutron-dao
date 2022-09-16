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
    TransferOwnership(String),
    AddAdmin (String),
    SubmitProposal(String, String)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    Config {},
}

pub type ConfigResponse = InstantiateMsg;

/// MsgTextPoposal defines a SDK message for submission of text proposal
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTextProposal {
    #[prost(bytes, tag = "1")]
    pub title: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes, tag = "1")]
    pub text: ::prost::alloc::vec::Vec<u8>,
}