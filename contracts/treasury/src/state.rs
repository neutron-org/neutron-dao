use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub distribution_rate: u8,
    pub distribution_contract: Addr,
    pub min_period: u64,
    pub denom: String,
    pub owner: Addr,
}

pub const TOTAL_RECEIVED: Item<Uint128> = Item::new("total_received");
pub const TOTAL_BANK_SPENT: Item<Uint128> = Item::new("total_bank_spent");
pub const TOTAL_DISTRIBUTED: Item<Uint128> = Item::new("total_distributed");

pub const LAST_GRAB_TIME: Item<u64> = Item::new("last_grab_time");
pub const LAST_BALANCE: Item<Uint128> = Item::new("last_balance");
pub const BANK_BALANCE: Item<Uint128> = Item::new("bank_balance");

pub const CONFIG: Item<Config> = Item::new("config");
