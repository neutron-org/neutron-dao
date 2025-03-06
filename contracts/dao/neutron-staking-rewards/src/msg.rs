use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub annual_reward_rate_bps: u64,
    pub blocks_per_year: u64,
    pub dao_address: String,
    pub staking_info_proxy: String,
    pub staking_denom: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the contract config (static parameters only).
    #[returns(ConfigResponse)]
    Config {},

    /// Returns just the state info (global reward index, last update).
    #[returns(StateResponse)]
    State {},

    /// Returns the user's current pending rewards.
    #[returns(RewardsResponse)]
    Rewards { user: String },
}

/// Response for `QueryMsg::Config`
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub dao_address: String,
    pub staking_info_proxy: String,
    pub annual_reward_rate_bps: u64,
    pub blocks_per_year: u64,
    pub staking_denom: String,
}

/// Response for `QueryMsg::State`
#[cw_serde]
pub struct StateResponse {
    pub global_reward_index: String,
    pub last_global_update_block: u64,
}

/// Response for `QueryMsg::Rewards`
#[cw_serde]
pub struct RewardsResponse {
    pub pending_rewards: Coin,
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
