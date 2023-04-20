use cwd_macros::{info_query, voting_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    // Credits contract address.
    pub credits_contract_address: String,
    // Description contains information that characterizes the vault.
    pub description: String,
    // Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        credits_contract_address: Option<String>,
        owner: String,
        description: Option<String>,
    },
}

#[voting_query]
#[info_query]
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Dao {},
    Description {},
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CreditsQueryMsg {
    BalanceAtHeight {
        address: String,
        height: Option<u64>,
    },

    TotalSupplyAtHeight {
        height: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
