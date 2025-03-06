use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};

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
}

/// Response for `QueryMsg::Providers`
#[cw_serde]
pub struct ProvidersResponse {
    pub providers: Vec<String>,
}
