use cwd_interface::Admin;
use cwd_macros::{info_query, voting_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    // Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: Option<Admin>,
    // Manager can update all configs except changing the owner. This will generally be an operations multisig for a DAO.
    pub manager: Option<String>,
    // A list of addresses of relative voting vault contracts.
    pub voting_vaults: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddVotingVault {
        new_voting_vault_contract: String,
    },
    RemoveVotingVault {
        old_voting_vault_contract: String,
    },
    UpdateConfig {
        owner: Option<String>,
        manager: Option<String>,
    },
}

#[voting_query]
#[info_query]
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Dao {},
    GetConfig {},
    VotingVaults {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct VotingVault {
    pub address: String,
    pub name: String,
    pub description: String,
}
