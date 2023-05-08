use cosmwasm_std::{Addr, CosmosMsg};
use cwd_pre_propose_base::msg::{ExecuteMsg as ExecuteBase, QueryMsg as QueryBase};
use neutron_sdk::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProposeMessage {
    ProposeOverrule {
        timelock_contract: String,
        proposal_id: u64,
    },
}

pub type ExecuteMsg = ExecuteBase<ProposeMessage>;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryExt {
    OverruleProposalId {
        timelock_address: String,
        subdao_proposal_id: u64,
    },
}

pub type QueryMsg = QueryBase<QueryExt>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MainDaoQueryMsg {
    GetSubDao { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubDao {
    /// The contract address of the SubDAO
    pub addr: String,
    /// The purpose/constitution for the SubDAO
    pub charter: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubdaoQueryMsg {
    /// Gets the contract's config. Returns Config.
    Config {},

    ProposalModules {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

/// Top level config type for core module.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubdaoConfig {
    /// The name of the contract.
    pub name: String,
    /// A description of the contract.
    pub description: String,
    /// The URI for the DAO as defined by the DAOstar standard
    /// https://daostar.one/EIP
    pub dao_uri: Option<String>,
    /// The address of the main DAO. It's capable of pausing and unpausing subDAO
    pub main_dao: Addr,
    /// The address of the DAO guardian. The security DAO is capable only of pausing the subDAO.
    pub security_dao: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
/// Top level type describing a proposal module.
pub struct SubdaoProposalModule {
    /// The address of the proposal module.
    pub address: Addr,
    /// The URL prefix of this proposal module as derived from the module ID.
    /// Prefixes are mapped to letters, e.g. 0 is 'A', and 26 is 'AA'.
    pub prefix: String,
    /// The status of the proposal module, e.g. 'Active' or 'Disabled.'
    pub status: SubdaoProposalModuleStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
/// The status of a proposal module.
pub enum SubdaoProposalModuleStatus {
    Enabled,
    Disabled,
}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalSingleQueryMsg {
    Dao {},
    ProposalCount {},
}
