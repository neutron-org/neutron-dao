use cosmwasm_std::{Addr, CosmosMsg, Timestamp};
use cwd_pre_propose_base::msg::{ExecuteMsg as ExecuteBase, QueryMsg as QueryBase};
use neutron_bindings::bindings::msg::NeutronMsg;
use neutron_subdao_pre_propose_single::types::ProposeMessage;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TimelockExecuteMsg {
    OverruleProposal { proposal_id: u64 },
}

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

// EXTERNAL TYPES SECTION BEGIN

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MainDaoQueryMsg {
    ListSubDaos {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubDao {
    /// The contract address of the SubDAO
    pub addr: String,
    /// The purpose/constitution for the SubDAO
    pub charter: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TimelockQueryMsg {
    /// Gets the config. Returns `state::Config`.
    Config {},

    /// Gets information about a proposal. Returns
    /// `proposals::Proposal`.
    Proposal { proposal_id: u64 },
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug, Eq, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct TimelockConfig {
    pub owner: Addr,
    pub timelock_duration: u64,
    // subDAO core module can timelock proposals.
    pub subdao: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DaoProposalQueryMsg {
    Dao {},
}

// EXTERNAL TYPES SECTION END
