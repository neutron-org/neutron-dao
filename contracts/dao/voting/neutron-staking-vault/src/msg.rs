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
    // Staking watcher contract.
    pub staking_tracker_contract_address: String,
    // Description contains information that characterizes the vault.
    pub description: String,
    // Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: String,
    // Name of the vault.
    pub name: String,
}

#[voting_vault]
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates config. Allowed only for owner to do.
    UpdateConfig {
        staking_tracker_contract_address: Option<String>,
        owner: Option<String>,
        description: Option<String>,
        name: Option<String>,
    },
    /// Adds given `addresses` to blacklist.
    AddToBlacklist { addresses: Vec<String> },
    /// Removes given `addresses` from blacklist.
    RemoveFromBlacklist {
        addresses: Vec<String>, // List of addresses to remove from the blacklist
    },
}

#[voting_query]
#[voting_vault_query]
#[info_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns contract's config.
    #[returns(crate::state::Config)]
    Config {},

    /// Lists blacklisted addresses.
    #[returns(Vec<Addr>)]
    ListBlacklistedAddresses {
        start_after: Option<Addr>,
        limit: Option<u32>,
    },

    // Returns true if given `address` is blacklisted.
    #[returns(bool)]
    IsAddressBlacklisted { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
