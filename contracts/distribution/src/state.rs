use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub denom: String,
    pub owner: Addr,
}
/// Map to store the amount of funds that are pending distribution to a given address
pub const PENDING_DISTRIBUTION: Map<Addr, Uint128> = Map::new("pending_distribution");
/// Map to store the amount of shares that a given address has
pub const SHARES: Map<Addr, Uint128> = Map::new("shares");

pub const CONFIG: Item<Config> = Item::new("config");

pub const FUND_COUNTER: Item<u64> = Item::new("fund_counter");
