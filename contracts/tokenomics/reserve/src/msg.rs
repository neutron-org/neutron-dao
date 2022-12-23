use cosmwasm_std::{Addr, Uint128};
use cwd_macros::{pausable, pausable_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: String,
    /// The address of the main DAO. It's capable of pausing and unpausing the contract
    pub main_dao_address: Addr,
    /// The address of the DAO guardian. The security DAO is capable only of pausing the contract.
    pub security_dao_address: Addr,
}

#[pausable]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),
    // Payout funds at DAO decision
    Payout {
        amount: Uint128,
        recipient: String,
    },
}

#[pausable_query]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configuration
    Config {},
}
