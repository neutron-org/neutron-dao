use cosmwasm_std::{CosmosMsg};
use cwd_interface::Admin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use neutron_bindings::bindings::msg::NeutronMsg;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    // Description contains information that characterizes the vault.
    pub description: String,
    // Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: Option<Admin>,
    // Manager can update all configs except changing the owner. This will generally be an operations multisig for a DAO.
    pub manager: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    TimelockProposal {},
    ExecuteProposal {},
    OverruleProposal {},
    UpdateConfig {
        owner: Option<String>,
        manager: Option<String>,
        description: Option<String>,
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

/// Information about a proposal returned by proposal queries.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ProposalResponse {
    /// The ID of the proposal being returned.
    pub id: u64,
    pub proposal: SingleChoiceProposal,
}

/// A list of proposals returned by `ListProposals` and
/// `ReverseProposals`.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ProposalListResponse {
    pub proposals: Vec<ProposalResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct SingleChoiceProposal {
    /// The ID of the proposal being returned.
    pub id: u64,

    /// The messages that will be executed should this proposal pass.
    pub msgs: Vec<CosmosMsg<NeutronMsg>>,
}