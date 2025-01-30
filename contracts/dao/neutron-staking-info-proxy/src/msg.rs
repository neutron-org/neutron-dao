use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub staking_rewards: Option<String>,
    pub staking_denom: String,
    pub providers: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Update contract configuration. Must be called by `owner`.
    UpdateConfig {
        owner: Option<String>,
        staking_rewards: Option<String>,
        staking_denom: Option<String>,
    },
    /// Update staking info providers. Must be called by `owner`
    UpdateProviders { providers: Option<Vec<String>> },
    /// Proxies update stake from set providers to the staking rewards contract.
    /// Must be called by one of the `PROVIDERS`.
    UpdateStake { user: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the contract config (static parameters only).
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ProvidersResponse)]
    Providers {},
    #[returns(Coin)]
    StakeQuery { user: String },
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

#[cw_serde]
pub struct MigrateMsg {}

/// Stake query to provider
#[cw_serde]
pub enum ProviderStakeQuery {
    /// Returns user stake from provider in Vec<Coin>
    User { address: String },
}
