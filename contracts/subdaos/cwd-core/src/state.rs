use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Top level config type for core module.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    /// The name of the contract.
    pub name: String,
    /// A description of the contract.
    pub description: String,
    /// The URI for the DAO as defined by the DAOstar standard
    /// https://daostar.one/EIP
    pub dao_uri: Option<String>,
    /// The address of the main DAO. It's capable of pausing and unpausing subDAO
    pub main_dao: Addr,
    /// The address of the DAO guardian. The security DAO is capable only of pausing the subDAO.
    pub security_dao: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
/// Top level type describing a proposal module.
pub struct ProposalModule {
    /// The address of the proposal module.
    pub address: Addr,
    /// The URL prefix of this proposal module as derived from the module ID.
    /// Prefixes are mapped to letters, e.g. 0 is 'A', and 26 is 'AA'.
    pub prefix: String,
    /// The status of the proposal module, e.g. 'Active' or 'Disabled.'
    pub status: ProposalModuleStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
/// The status of a proposal module.
pub enum ProposalModuleStatus {
    Enabled,
    Disabled,
}

/// The current configuration of the module.
pub const CONFIG: Item<Config> = Item::new("config_v2");

/// The height the subDAO is paused until. If it's None, the subDAO is not paused.
pub const PAUSED_UNTIL: Item<Option<u64>> = Item::new("paused_until");

/// The voting module associated with this contract.
pub const VOTING_REGISTRY_MODULE: Item<Addr> = Item::new("voting_module");

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
