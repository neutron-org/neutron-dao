use cw_storage_plus::{Item, Map};
use neutron_subdao_timelock_single::types::{Config, SingleChoiceProposal};

/// Default limit for proposal pagination.
pub const DEFAULT_LIMIT: u64 = 30;

pub const CONFIG: Item<Config> = Item::new("config");
pub const PROPOSALS: Map<u64, SingleChoiceProposal> = Map::new("proposals");
/// Execution errors for proposals that do not close on failure (Config.close_proposal_on_execution_failure set to false)
pub const PROPOSAL_EXECUTION_ERRORS: Map<u64, String> = Map::new("proposal_execution_errors");
