use cosmwasm_std::Uint128;
use cwd_interface::Admin;
use cwd_macros::{info_query, voting_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    // Description contains information that characterizes the vault.
    pub description: String,
    // Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: Option<Admin>,
    // Manager can update all configs except changing the owner. This will generally be an operations multisig for a DAO.
    pub manager: Option<String>,
    // Token denom e.g. untrn, or some ibc denom
    pub denom: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Bond {},
    Unbond {
        amount: Uint128,
    },
    UpdateConfig {
        owner: Option<String>,
        manager: Option<String>,
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
    GetConfig {},
    ListBonders {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ListBondersResponse {
    pub bonders: Vec<BonderBalanceResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BonderBalanceResponse {
    pub address: String,
    pub balance: Uint128,
}
