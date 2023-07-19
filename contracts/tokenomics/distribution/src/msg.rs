use cosmwasm_std::Uint128;
use cwd_macros::{pausable, pausable_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// Denom used for rewards distribution. All funds in any other denoms will be ignored.
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

    /// Alter shareholder's weights
    SetShares { shares: Vec<(String, Uint128)> },

    /// Distribute funds between share holders. It is called from reserve contract only
    /// when part of the fund is going to distribution between share holders.
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
    /// List of pending funds to addresses (to be distributed); returns [`Vec<(Addr, Uint128)>`]
    Pending {},
    /// List of current shareholder weights; returns [`Vec<(Addr, Uint128)>`]
    Shares {},
}
