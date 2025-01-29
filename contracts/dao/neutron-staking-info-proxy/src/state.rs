use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Configuration.
#[cw_serde]
pub struct Config {
    /// owner can update contract's config
    pub owner: Addr,
    /// contract address which we proxy to.
    pub staking_rewards: Option<Addr>,
    /// denom in which staking rewards work.
    pub staking_denom: String,
}

/// List of providers from which to query staking info
pub const PROVIDERS: Map<Addr, ()> = Map::new("providers");

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
