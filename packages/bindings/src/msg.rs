use cosmwasm_std::{CosmosMsg, CustomMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// A number of Custom messages that can call into the Neutron bindings
pub enum NeutronMsg {
    /// AddAdmin registers an interchain account on remote chain
    AddAdmin {
        admin: String,
    },
    SubmitProposal {
        proposals: Proposals,
    },
}

/// MsgTextPoposal defines a SDK message for submission of text proposal
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TextProposal {
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proposals {
    pub text_proposal: Option<TextProposal>,
    pub param_change_proposal: Option<ParamChangeProposal>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamChangeProposal {
    pub title: String,
    pub description: String,
    pub param_changes: Vec<ParamChange>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamChange {
    pub subspace: String,
    pub key: String,
    pub value: String,
}

impl NeutronMsg {
    pub fn add_admin(admin: String) -> Self {
        NeutronMsg::AddAdmin { admin }
    }

    pub fn submit_text_proposal(proposal: TextProposal) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {
                text_proposal: Option::from(proposal),
                param_change_proposal: None,
            },
        }
    }
    pub fn submit_param_change_proposal(proposal: ParamChangeProposal) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {
                text_proposal: None,
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
