use cwd_interface::Admin;
use cwd_macros::{info_query, voting_query, voting_vault, voting_vault_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    /// Name contains the vault name which is used to ease the vault's recognition.
    pub name: String,
    // Description contains information that characterizes the vault.
    pub description: String,
    // Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: Admin,
    // Manager can update all configs except changing the owner. This will generally be an operations multisig for a DAO.
    pub manager: Option<String>,
    // Token denom e.g. untrn, or some ibc denom
    pub denom: String,
}

#[voting_vault]
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        name: String,
        description: String,
        owner: String,
        manager: Option<String>,
    },
}

#[voting_query]
#[voting_vault_query]
#[info_query]
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
