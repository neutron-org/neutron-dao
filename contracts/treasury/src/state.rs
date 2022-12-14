use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub distribution_rate: u8,
    pub distribution_contract: Addr,
    pub reserve_contract: Addr,
    pub min_period: u64,
    pub denom: String,
    pub owner: Addr,
}

pub const TOTAL_RECEIVED: Item<Uint128> = Item::new("total_received");
pub const TOTAL_DISTRIBUTED: Item<Uint128> = Item::new("total_distributed");
pub const TOTAL_RESERVED: Item<Uint128> = Item::new("total_reserved");

pub const LAST_DISTRIBUTION_TIME: Item<u64> = Item::new("last_grab_time");

pub const CONFIG: Item<Config> = Item::new("config");
