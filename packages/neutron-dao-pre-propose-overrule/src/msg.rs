use crate::types::ProposeMessage;
use cosmwasm_std::CosmosMsg;
use cwd_pre_propose_base::msg::{ExecuteMsg as ExecuteBase, QueryMsg as QueryBase};
use neutron_bindings::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type ExecuteMsg = ExecuteBase<ProposeMessage>;
pub type QueryMsg = QueryBase;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub main_dao: String,
}

/// Internal version of the propose message that includes the
/// `proposer` field. The module will fill this in based on the sender
/// of the external message.
#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProposeMessageInternal {
    Propose {
        title: String,
        description: String,
        msgs: Vec<CosmosMsg<NeutronMsg>>,
        proposer: Option<String>,
    },
}
