use cosmwasm_std::Uint128;
use cwd_macros::{pausable, pausable_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: String,
    /// The address of the main DAO. It's capable of pausing and unpausing the contract
    pub main_dao_address: String,
    /// The address of the DAO guardian. The security DAO is capable only of pausing the contract.
    pub security_dao_address: String,
}

#[pausable]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),

    SetShares {
        shares: Vec<(String, Uint128)>,
    },

    /// Distribute funds between share holders. It is called from treasury contract only
    /// when part of the fund is going to distribution betrween share holders.
    Fund {},

    /// Claim the funds that have been distributed to the contract's account
    Claim {},
}

#[pausable_query]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    Config {},
    Pending {},
    Shares {},
}
