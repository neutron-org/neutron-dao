use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub distribution_rate: u8,
    pub min_time_elapsed_between_fundings: u64,
    pub denom: String,
    pub owner: Addr,
    pub dao: Addr,
}

pub const TOTAL_RECEIVED: Item<Uint128> = Item::new("total_received");
pub const TOTAL_BANK_SPENT: Item<Uint128> = Item::new("total_bank_spent");
pub const TOTAL_DISTRIBUTED: Item<Uint128> = Item::new("total_distributed");

pub const LAST_BALANCE: Item<Uint128> = Item::new("last_balance");
pub const DISTRIBUTION_BALANCE: Item<Uint128> = Item::new("distribution_balance");
pub const PENDING_DISTRIBUTION: Map<&[u8], Uint128> = Map::new("pending_distribution");
pub const BANK_BALANCE: Item<Uint128> = Item::new("bank_balance");

pub const SHARES: Map<&[u8], Uint128> = Map::new("shares");

pub const CONFIG: Item<Config> = Item::new("config");
