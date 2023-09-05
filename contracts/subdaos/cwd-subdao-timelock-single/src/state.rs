use cw_storage_plus::{Item, Map};
use neutron_subdao_timelock_single::types::{Config, FailedExecutionError, SingleChoiceProposal};

/// Default limit for proposal pagination.
pub const DEFAULT_LIMIT: u64 = 30;

pub const CONFIG: Item<Config> = Item::new("config");
pub const PROPOSALS: Map<u64, SingleChoiceProposal> = Map::new("proposals");
pub const PROPOSAL_FAILED_EXECUTION_ERRORS: Map<u64, Vec<FailedExecutionError>> =
    Map::new("failed_proposal_errors");
