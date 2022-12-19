use cosmwasm_std::CosmosMsg;
use neutron_bindings::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cwd_interface::Admin;
use crate::proposal::SingleChoiceProposal;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    // Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: Option<Admin>,

    // Timelock duration for all proposals (starts when TimelockProposal message handler is executed).
    // In seconds.
    pub timelock_duration: u64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    TimelockProposal {
        proposal_id: u64,
        msgs: Vec<CosmosMsg<NeutronMsg>>,
    },
    ExecuteProposal { proposal_id: u64 },
    OverruleProposal { proposal_id: u64 },
    UpdateConfig {
        owner: Option<String>,
        timelock_duration: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Gets the config. Returns `state::Config`.
    Config {},

    /// Gets information about a proposal. Returns
    /// `proposals::Proposal`.
    Proposal { proposal_id: u64 },

    /// Lists all the proposals that have been cast in this
    /// module. Returns `query::ProposalListResponse`.
    ListProposals {
        /// The proposal ID to start listing proposals after. For
        /// example, if this is set to 2 proposals with IDs 3 and
        /// higher will be returned.
        start_after: Option<u64>,
        /// The maximum number of proposals to return as part of this
        /// query. If no limit is set a max of 30 proposals will be
        /// returned.
        limit: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

/// A list of proposals returned by `ListProposals`.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ProposalListResponse {
    pub proposals: Vec<SingleChoiceProposal>,
}