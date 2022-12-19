use cosmwasm_std::{CosmosMsg, Timestamp};
use neutron_bindings::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct SingleChoiceProposal {
    /// The ID of the proposal being returned.
    pub id: u64,

    /// The timestamp at which the proposal was submitted to the timelock contract.
    pub timelock_ts: Timestamp,

    /// The messages that will be executed should this proposal be executed.
    pub msgs: Vec<CosmosMsg<NeutronMsg>>,

    pub status: ProposalStatus,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Copy)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ProposalStatus {
    /// The proposal is open for voting.
    Timelocked,
    /// The proposal has been overruled.
    Overruled,
    /// The proposal has been executed.
    Executed,
    /// The proposal's execution failed.
    ExecutionFailed,
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Timelocked => write!(f, "timelocked"),
            ProposalStatus::Overruled => write!(f, "overruled"),
            ProposalStatus::Executed => write!(f, "executed"),
            ProposalStatus::ExecutionFailed => write!(f, "execution_failed"),
        }
    }
}
