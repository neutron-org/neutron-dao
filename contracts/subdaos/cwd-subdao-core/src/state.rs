use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use neutron_subdao_core::types::{Config, ProposalModule};

/// The current configuration of the module.
pub const CONFIG: Item<Config> = Item::new("config_v2");

/// The height the subDAO is paused until. If it's None, the subDAO is not paused.
pub const PAUSED_UNTIL: Item<Option<u64>> = Item::new("paused_until");

/// The voting module associated with this contract.
pub const VOTE_MODULE: Item<Addr> = Item::new("voting_module");

/// The proposal modules associated with this contract.
/// When we change the data format of this map, we update the key (previously "proposal_modules")
/// to create a new namespace for the changed state.
pub const PROPOSAL_MODULES: Map<Addr, ProposalModule> = Map::new("proposal_modules_v2");

/// The count of active proposal modules associated with this contract.
pub const ACTIVE_PROPOSAL_MODULE_COUNT: Item<u32> = Item::new("active_proposal_module_count");

/// The count of total proposal modules associated with this contract.
pub const TOTAL_PROPOSAL_MODULE_COUNT: Item<u32> = Item::new("total_proposal_module_count");

// General purpose KV store for DAO associated state.
pub const ITEMS: Map<String, String> = Map::new("items");

/// List of SubDAOs associated to this DAO. Each SubDAO has an optional charter.
pub const SUBDAO_LIST: Map<&Addr, Option<String>> = Map::new("sub_daos");
