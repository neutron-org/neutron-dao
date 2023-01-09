use cosmwasm_std::CosmosMsg;
use neutron_bindings::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
        timelock_duration: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalQueryMsg {
    /// Returns the number of proposals that have been created in this
    /// module.
    ProposalCount {},
}
