use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// The contract's owner
    pub owner: String,
    /// Locked denom
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership {
        new_owner: String,
    },
    LockFunds {},
    UnlockFunds {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    Config {},
    /// Amount of NTRN tokens of a voting recipient currently locked in the contract;
    /// returns [`VotingPowerResponse`]
    VotingPower { user: String },
    /// Enumerate all voting recipients and return their current voting power;
    /// returns [`Vec`] of [`VotingPowerResponse`]'s
    VotingPowers {},
}

pub type ConfigResponse = InstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct VotingPowerResponse {
    /// Address of the user
    pub user: String,
    /// The user's current voting power, i.e. the amount of NTRN tokens locked in voting contract
    pub voting_power: Uint128,
}
