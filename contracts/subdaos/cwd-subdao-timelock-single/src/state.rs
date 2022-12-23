use crate::proposal::SingleChoiceProposal;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub owner: Option<Addr>,
    pub timelock_duration: Option<u64>,
    // subDAO core module can timelock proposals.
    pub subdao: Option<Addr>,
}

/// Default limit for proposal pagination.
pub const DEFAULT_LIMIT: u64 = 30;

pub const CONFIG: Item<Config> = Item::new("config");
pub const PROPOSALS: Map<u64, SingleChoiceProposal> = Map::new("proposals");
