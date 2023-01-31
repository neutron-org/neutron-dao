use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, SnapshotItem, SnapshotMap, Strategy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub description: String,
    pub owner: Option<Addr>,
    pub manager: Option<Addr>,
    pub denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");
pub const BONDED_BALANCES: SnapshotMap<&Addr, Uint128> = SnapshotMap::new(
    "bonded_balances",
    "bonded_balance__checkpoints",
    "bonded_balance__changelog",
    Strategy::EveryBlock,
);

pub const BONDED_TOTAL: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_bonded",
    "total_bonded__checkpoints",
    "total_bonded__changelog",
    Strategy::EveryBlock,
);
