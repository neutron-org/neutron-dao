use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The contract's owner
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),
    InitVoting(),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations; returns `ConfigResponse`
    Config {},
    /// Amount of NTRN tokens of a voting recipient current locked in the contract; returns `VotingPowerResponse`
    VotingPower { user: String },
    /// Enumerate all voting recipients and return their current voting power; returns `Vec<VotingPowerResponse>`
    VotingPowers {},
}

pub type ConfigResponse = InstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotingPowerResponse {
    /// Address of the user
    pub user: String,
    /// The user's current voting power, i.e. the amount of NTRN tokens locked in voting contract
    pub voting_power: Uint128,
}
