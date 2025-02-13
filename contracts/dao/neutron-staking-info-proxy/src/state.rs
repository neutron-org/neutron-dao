use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Configuration.
#[cw_serde]
pub struct Config {
    /// The owner can update the contract's configuration and providers.
    pub owner: Addr,
    /// The contract address to which requests are proxied.
    pub staking_rewards: Option<Addr>,
    /// The denom used for staking rewards.
    pub staking_denom: String,
}

/// List of providers for querying staking information.
/// A provider is a contract that supplies stake information updates.
pub const PROVIDERS: Map<Addr, ()> = Map::new("providers");

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        Ok(())
    }
}

/// Contract's configuration parameters.
pub const CONFIG: Item<Config> = Item::new("config");
