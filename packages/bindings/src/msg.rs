use cosmwasm_std::{CosmosMsg, CustomMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// A number of Custom messages that can call into the Neutron bindings
pub enum NeutronMsg {
    /// TODO: add anotation
    /// TODO: move this to neutron sdk; bring neutron-sdk instead to this project
    /// This message can be sent only by neutron dao (cuz only neutron dao has admin rights)
    SubmitProposal { proposals: Proposals },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proposals {
    pub param_change_proposal: Option<ParamChangeProposal>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamChangeProposal {
    pub title: String,
    pub description: String,
    pub param_changes: Vec<ParamChange>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamChange {
    pub subspace: String,
    pub key: String,
    pub value: String,
}

impl NeutronMsg {
    pub fn submit_param_change_proposal(proposal: ParamChangeProposal) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {
                param_change_proposal: Option::from(proposal),
            },
        }
    }
}

impl From<NeutronMsg> for CosmosMsg<NeutronMsg> {
    fn from(msg: NeutronMsg) -> CosmosMsg<NeutronMsg> {
        CosmosMsg::Custom(msg)
    }
}

impl CustomMsg for NeutronMsg {}
