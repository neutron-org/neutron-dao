use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum ExecuteMsg {
    /// Update contract configuration. Must be called by `owner`.
    UpdateConfig {
        owner: Option<String>,
        annual_reward_rate_bps: Option<u64>,
        blocks_per_year: Option<u64>,
        staking_info_proxy: Option<String>,
        staking_denom: Option<String>,
    },
    /// Called by the (authorized) Staking Info Proxy whenever a user’s stake changes.
    UpdateStake { user: String },
    /// Called by the (authorized) Staking Info Proxy whenever a validator gets slashed.
    Slashing {},
    /// Called by a user to claim their accrued rewards. Allows to specify an optional
    /// address to which the rewards should be sent.
    ClaimRewards { to_address: Option<String> },
}
