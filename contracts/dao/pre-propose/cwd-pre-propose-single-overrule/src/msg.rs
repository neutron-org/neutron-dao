use cosmwasm_std::CosmosMsg;
use neutron_bindings::bindings::msg::NeutronMsg;

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProposeMessage {
    ProposeOverrule {
        timelock_contract: String,
        proposal_id: u64,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TimelockExecuteMsg {
    OverruleProposal { proposal_id: u64 },
}

pub type ExecuteMsg = ExecuteBase<ProposeMessage>;
pub type QueryMsg = QueryBase;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

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
