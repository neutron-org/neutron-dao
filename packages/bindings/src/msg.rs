use std::ptr::null;
use cosmwasm_std::{CosmosMsg, CustomMsg};
use schemars::gen::SchemaGenerator;
use schemars::JsonSchema;
use schemars::schema::Schema;
use serde::{Deserialize, Serialize};
use crate::ProtobufAny;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// A number of Custom messages that can call into the Neutron bindings
pub enum NeutronMsg {
    /// AddAdmin registers an interchain account on remote chain
    AddAdmin {
        admin: String,
    },
    SubmitProposal{
        proposals: Proposals
    }
}

/// MsgTextPoposal defines a SDK message for submission of text proposal
#[derive(Clone, PartialEq)]
pub struct TextProposal {
    pub title: String,
    pub description: String,
}

pub struct Proposals {
    pub text_proposal: TextProposal,
    // pub param_change_proposal: ParamChangeProposal
}

// pub struct ParamChangeProposal {
//     pub title: String,
//     pub description: String,
// }

impl NeutronMsg {
    pub fn add_admin(
        admin: String,
    ) -> Self {
        NeutronMsg::AddAdmin {
            admin
        }
    }

    pub fn submit_text_proposal(
        proposal: TextProposal,
    ) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {text_proposal: proposal}
        }
    }
}

impl From<NeutronMsg> for CosmosMsg<NeutronMsg> {
    fn from(msg: NeutronMsg) -> CosmosMsg<NeutronMsg> {
        CosmosMsg::Custom(msg)
    }
}

impl CustomMsg for NeutronMsg {}

