use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const TOKENS_LOCKED: Map<&Addr, Uint128> = Map::new("positions");

pub const OWNER: Item<Addr> = Item::new("owner");
