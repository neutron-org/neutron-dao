use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a response model for voting vault's ListBonders calls.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ListBondersResponse {
    pub bonders: Vec<BonderBalanceResponse>,
}

/// Represents a single bonder balance model for voting vault's ListBonders calls.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BonderBalanceResponse {
    pub address: String,
    pub balance: Uint128,
}
