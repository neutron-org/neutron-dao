use cwd_pre_propose_base::msg::{ExecuteMsg as ExecuteBase, QueryMsg as QueryBase};
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
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryExt {
    OverruleProposalId {
        timelock_address: String,
        subdao_proposal_id: u64,
    },
}

pub type QueryMsg = QueryBase<QueryExt>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
