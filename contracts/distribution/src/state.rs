use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub denom: String,
    pub owner: Addr,
}

pub const PENDING_DISTRIBUTION: Map<&[u8], Uint128> = Map::new("pending_distribution");

pub const SHARES: Map<&[u8], Uint128> = Map::new("shares");

pub const CONFIG: Item<Config> = Item::new("config");

pub const FUND_COUNTER: Item<u64> = Item::new("fund_counter");
