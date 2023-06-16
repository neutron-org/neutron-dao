use cwd_macros::{info_query, voting_query, voting_vault, voting_vault_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    /// Name contains the vault name which is used to ease the vault's recognition.
    pub name: String,
    /// Description contains information that characterizes the vault.
    pub description: String,
    /// The ATOM Vesting LP contract behind the vault.
    pub atom_vesting_lp_contract: String,
    /// The ATOM/NTRN CL pool contract.
    pub atom_cl_pool_contract: String,
    /// The USDC Vesting LP contract behind the vault.
    pub usdc_vesting_lp_contract: String,
    /// The USDC/NTRN CL pool oracle contract.
    pub usdc_cl_pool_contract: String,
    /// Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: String,
}

#[voting_vault]
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: String,
        atom_vesting_lp_contract: String,
        atom_cl_pool_contract: String,
        usdc_vesting_lp_contract: String,
        usdc_cl_pool_contract: String,
        name: String,
        description: String,
    },
}

#[voting_query]
#[voting_vault_query]
#[info_query]
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {
    pub atom_cl_pool: String,
    pub usdc_cl_pool: String,
}
