use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cwd_macros::{pausable, pausable_query};
use exec_control::pause::PauseInfoResponse;
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

    /// Distribute funds between share holders. It is called from reserve contract only
    /// when part of the fund is going to distribution betrween share holders.
    Fund {},

    /// Claim the funds that have been distributed to the contract's account
    Claim {},
}

#[pausable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    #[returns(crate::state::Config)]
    Config {},
    #[returns(Vec<(Addr, Uint128)>)]
    Pending {},
    #[returns(Vec<(Addr, Uint128)>)]
    Shares {},
}
