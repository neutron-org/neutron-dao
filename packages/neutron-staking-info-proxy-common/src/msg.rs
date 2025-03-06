use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum ExecuteMsg {
    /// Updates the contract configuration. Must be called by the `owner`.
    UpdateConfig {
        owner: Option<String>,
        staking_rewards: Option<String>,
        staking_denom: Option<String>,
    },
    /// Updates staking info providers. Must be called by the `owner`.
    UpdateProviders { providers: Vec<String> },
    /// Proxies stake updates from designated providers to the staking rewards contract.
    /// Must be called by one of the `PROVIDERS`.
    UpdateStake { user: String },
    /// Proxies slashing evens from designated providers to the staking rewards contract.
    /// Must be called by one of the `PROVIDERS`.
    Slashing {},
}
