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
        content: ProtobufAny
    }
}

impl NeutronMsg {
    pub fn add_admin(
        admin: String,
    ) -> Self {
        NeutronMsg::AddAdmin {
            admin
        }
    }

    pub fn submit_proposal(
        content: ProtobufAny,
    ) -> Self {
        NeutronMsg::SubmitProposal {
            content
        }
    }
}

impl From<NeutronMsg> for CosmosMsg<NeutronMsg> {
    fn from(msg: NeutronMsg) -> CosmosMsg<NeutronMsg> {
        CosmosMsg::Custom(msg)
    }
}

impl CustomMsg for NeutronMsg {}

