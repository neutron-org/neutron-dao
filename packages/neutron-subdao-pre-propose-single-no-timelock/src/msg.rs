use cw_utils::Duration;
use cwd_interface::ModuleInstantiateInfo;
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


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Pauses the contract for a set duration.
    /// When paused the DAO is unable to execute proposals
    Pause { duration: Duration },

}
