use cosmwasm_std::{Decimal, Uint128};
use cwd_macros::pausable;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub main_dao_address: String,
    pub denom: String,
    /// Distribution rate (0-1) which goes to distribution contract
    pub distribution_rate: Decimal,
    /// Minimum period between distribution calls
    pub min_period: u64,
    /// Address of distribution contract
    pub distribution_contract: String,
    /// Address of reserve contract
    pub reserve_contract: String,
    /// Address of security DAO contract
    pub security_dao_address: String,
}

#[pausable]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),

    /// Distribute pending funds between Bank and Distribution accounts
    Distribute {},

    // //Update config
    UpdateConfig {
        distribution_rate: Option<Decimal>,
        min_period: Option<u64>,
        distribution_contract: Option<String>,
        reserve_contract: Option<String>,
        security_dao_address: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    Config {},
    Stats {},
    /// Returns information about if the contract is currently paused.
    PauseInfo {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StatsResponse {
    pub total_received: Uint128,
    pub total_distributed: Uint128,
    pub total_reserved: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributeMsg {
    Fund {},
}

/// Information about if the contract is currently paused.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum PauseInfoResponse {
    Paused { until_height: u64 },
    Unpaused {},
}
