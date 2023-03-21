use cosmwasm_std::CosmosMsg;
use cwd_pre_propose_base::msg::{ExecuteMsg as ExecuteBase, QueryMsg as QueryBase};
use neutron_bindings::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProposeMessage {
    ProposeOverrule {
        timelock_contract: String,
        proposal_id: u64,
    },
}

pub type ExecuteMsg = ExecuteBase<ProposeMessage>;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub main_dao: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryExt {
    OverruleProposalId {
        timelock_address: String,
        subdao_proposal_id: u64,
    },
}

pub type QueryMsg = QueryBase<QueryExt>;
