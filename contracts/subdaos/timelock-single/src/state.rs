use cosmwasm_std::{Addr};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::msg::SingleChoiceProposal;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub description: String,
    pub owner: Option<Addr>,
    pub manager: Option<Addr>,
}

/// Default limit for proposal pagination.
pub const DEFAULT_LIMIT: u64 = 30;

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");
pub const PROPOSALS: Map<u64, SingleChoiceProposal> = Map::new("proposals");