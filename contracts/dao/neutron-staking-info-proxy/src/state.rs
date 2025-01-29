use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// Configuration.
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub staking_vault: Addr,
}

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
