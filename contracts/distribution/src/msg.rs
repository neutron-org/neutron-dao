use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// The contract's owner
    pub owner: String,
    pub dao: String,
    pub denom: String,
    pub distribution_rate: u8,
    pub min_time_elapsed_between_distributions: u64,
    pub min_time_elapsed_between_grabs: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),

    SetShares {
        shares: Vec<(String, Uint128)>,
    },

    /// Distribute funds to the and distribution accounts according to their shares
    Distribute(),

    /// Distribute pending funds between Bank and Distribution accounts
    Grab(),

    // Payout funds at DAO decision
    Payout {
        amount: Uint128,
        recipient: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    Config {},
    Stats {},
    Shares {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StatsResponse {
    pub total_received: Uint128,
    pub total_bank_spent: Uint128,
    pub total_distributed: Uint128,
    pub bank_balance: Uint128,
    pub distribution_balance: Uint128,
    pub last_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ShareResponse {
    address: Addr,
    shares: Uint128,
}
