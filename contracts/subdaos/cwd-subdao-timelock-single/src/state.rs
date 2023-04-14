use cw_storage_plus::{Item, Map};
use neutron_subdao_timelock_single::types::{Config, SingleChoiceProposal};

/// Default limit for proposal pagination.
pub const DEFAULT_LIMIT: u64 = 30;

pub const CONFIG_OLD: Item<Config> = Item::new("config");
pub const CONFIG: Item<Config> = Item::new("configv2");
pub const PROPOSALS: Map<u64, SingleChoiceProposal> = Map::new("proposals");
