use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::CosmosMsg;
use neutron_sdk::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    // Overrule pre proposal module from the main DAO
    pub overrule_pre_propose: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    TimelockProposal {
        proposal_id: u64,
        msgs: Vec<CosmosMsg<NeutronMsg>>,
    },
    ExecuteProposal {
        proposal_id: u64,
    },
    OverruleProposal {
        proposal_id: u64,
    },
    UpdateConfig {
        owner: Option<String>,
        overrule_pre_propose: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the config. Returns `types::Config`.
    #[returns(crate::types::Config)]
    Config {},

    /// Gets information about a proposal. Returns
    /// `proposals::Proposal`.
    #[returns(crate::types::SingleChoiceProposal)]
    Proposal { proposal_id: u64 },

    /// Lists all the proposals that have been cast in this
    /// module. Returns `query::ProposalListResponse`.
    #[returns(crate::types::ProposalListResponse)]
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
