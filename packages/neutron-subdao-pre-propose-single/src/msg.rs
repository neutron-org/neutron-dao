use crate::types::ProposeMessage;
use cwd_interface::ModuleInstantiateInfo;
use cwd_pre_propose_base::msg::{ExecuteMsg as ExecuteBase, QueryMsg as QueryBase};
use cwd_voting::deposit::UncheckedDepositInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Information about the deposit requirements for this
    /// module. None if no deposit.
    pub deposit_info: Option<UncheckedDepositInfo>,
    /// If false, only members (addresses with voting power) may create
    /// proposals in the DAO. Otherwise, any address may create a
    /// proposal so long as they pay the deposit.
    pub open_proposal_submission: bool,

    /// Instantiate information for timelock module.
    pub timelock_module_instantiate_info: ModuleInstantiateInfo,
}

pub type ExecuteMsg = ExecuteBase<ProposeMessage>;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryExt {
    TimelockAddress {},
}

pub type QueryMsg = QueryBase<QueryExt>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
