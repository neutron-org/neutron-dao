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
        proposals: Proposals
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
    pub community_spend_proposal: Option<CommunitySpendProposal>,
    pub client_update_spend_proposal: Option<ClientUpdateSpendProposal>,
    pub software_update_proposal: Option<SoftwareUpdateProposal>,
    pub cancel_software_update_proposal: Option<CancelSoftwareUpdateProposal>
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
pub struct SoftwareUpdateProposal {
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CancelSoftwareUpdateProposal {
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CommunitySpendProposal {
    pub title: String,
    pub description: String,
    pub recipient: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClientUpdateSpendProposal {
    pub title: String,
    pub description: String,
    pub subject_client_id: String,
    pub substitute_client_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamChange {
    pub subspace: String,
    pub key: String,
    pub value: String,
}

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
            proposals: Proposals {
                text_proposal: Option::from(proposal),
                param_change_proposal: Option::from(None),
                community_spend_proposal: Option::from(None),
                client_update_spend_proposal: Option::from(None),
                software_update_proposal: Option::from(None),
                cancel_software_update_proposal: Option::from(None) }
        }
    }
    pub fn submit_param_change_proposal(
        proposal: ParamChangeProposal,
    ) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {
                text_proposal: Option::from(None),
                param_change_proposal: Option::from(proposal),
                community_spend_proposal: Option::from(None),
                client_update_spend_proposal: Option::from(None),
                software_update_proposal: Option::from(None),
                cancel_software_update_proposal: Option::from(None)}
        }
    }
    pub fn submit_community_spend_proposal(
        proposal: CommunitySpendProposal,
    ) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {
                text_proposal: Option::from(None),
                param_change_proposal: Option::from(None),
                community_spend_proposal: Option::from(proposal),
                client_update_spend_proposal: Option::from(None),
                software_update_proposal: Option::from(None),
                cancel_software_update_proposal: Option::from(None) }
        }
    }

    pub fn submiit_client_update_spend_proposal(
        proposal: ClientUpdateSpendProposal,
    ) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {
                text_proposal: Option::from(None),
                param_change_proposal: Option::from(None),
                community_spend_proposal: Option::from(None),
                client_update_spend_proposal: Option::from(proposal),
                software_update_proposal: Option::from(None),
                cancel_software_update_proposal: Option::from(None) }
        }
    }

    pub fn submit_software_update_proposal(
        proposal: SoftwareUpdateProposal,
    ) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {
                text_proposal: Option::from(None),
                param_change_proposal: Option::from(None),
                community_spend_proposal: Option::from(None),
                client_update_spend_proposal: Option::from(None),
                software_update_proposal: Option::from(proposal),
                cancel_software_update_proposal: Option::from(None) }
        }
    }

    pub fn submit_cancel_software_update_proposal(
        proposal: CancelSoftwareUpdateProposal,
    ) -> Self {
        NeutronMsg::SubmitProposal {
            proposals: Proposals {
                text_proposal: Option::from(None),
                param_change_proposal: Option::from(None),
                community_spend_proposal: Option::from(None),
                client_update_spend_proposal: Option::from(None),
                software_update_proposal: Option::from(None),
                cancel_software_update_proposal: Option::from(proposal) }
        }
    }

}

impl From<NeutronMsg> for CosmosMsg<NeutronMsg> {
    fn from(msg: NeutronMsg) -> CosmosMsg<NeutronMsg> {
        CosmosMsg::Custom(msg)
    }
}

impl CustomMsg for NeutronMsg {}

