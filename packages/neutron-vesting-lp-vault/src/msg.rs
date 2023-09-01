use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cwd_interface::voting::{
    BondingStatusResponse, InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
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
    /// The ATOM oracle contract behind the vault.
    pub atom_oracle_contract: String,
    /// The USDC Vesting LP contract behind the vault.
    pub usdc_vesting_lp_contract: String,
    /// The USDC oracle contract behind the vault.
    pub usdc_oracle_contract: String,
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
        atom_oracle_contract: String,
        usdc_vesting_lp_contract: String,
        usdc_oracle_contract: String,
        name: String,
        description: String,
    },
}

#[voting_query]
#[voting_vault_query]
#[info_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::types::Config)]
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
