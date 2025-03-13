use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    // Use an Option to allow setting this value later in case of a cyclical dependency.
    pub staking_rewards: Option<String>,
    pub staking_denom: String,
    pub providers: Vec<String>,
}

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

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieves the contract configuration.
    #[returns(ConfigResponse)]
    Config {},
    /// Retrieves the list of providers.
    #[returns(ProvidersResponse)]
    Providers {},
    /// Retrieves the user's stake summed across all providers, filtered by `config.staking_denom`.
    #[returns(Coin)]
    UserStake { address: String, height: u64 },
}

/// Queries that each staking provider must implement.
#[cw_serde]
#[derive(QueryResponses)]
pub enum ProviderStakeQueryMsg {
    /// Gets the staked (bonded) tokens for given `address` at given `height`.
    /// Stake of unbonded validators does not count.
    /// If height is None, latest block stake info will be issued.
    #[returns(Uint128)]
    StakeAtHeight {
        address: String,
        height: Option<u64>,
    },

    /// Gets the total staked (bonded) tokens for given `height`.
    /// Stake of unbonded validators does not count.
    #[returns(Uint128)]
    TotalStakeAtHeight { height: Option<u64> },
}

/// Response for `QueryMsg::Config`
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub staking_rewards: Option<String>,
    pub staking_denom: String,
}

/// Response for `QueryMsg::Providers`
#[cw_serde]
pub struct ProvidersResponse {
    pub providers: Vec<String>,
}
