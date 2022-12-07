use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub denom: String,
    pub distribution_rate: u8,
    pub min_period: u64,
    pub distribution_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),

    /// Distribute pending funds between Bank and Distribution accounts
    Distribute {},

    // Payout funds at DAO decision
    Payout {
        amount: Uint128,
        recipient: String,
    },
    // //Update config
    UpdateConfig {
        distribution_rate: Option<u8>,
        min_period: Option<u64>,
        distribution_contract: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    Config {},
    Stats {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StatsResponse {
    pub total_received: Uint128,
    pub total_bank_spent: Uint128,
    pub total_distributed: Uint128,
    pub bank_balance: Uint128,
    pub last_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributeMsg {
    Fund {},
}
