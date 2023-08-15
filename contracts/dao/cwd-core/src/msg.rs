use crate::query::{DumpStateResponse, GetItemResponse, PauseInfoResponse};
use crate::state::ProposalModule;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg};
use cw_utils::Duration;
use cwd_interface::voting::{
    InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use cwd_interface::ModuleInstantiateInfo;
use cwd_macros::{info_query, voting_query};
use neutron_sdk::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::query::SubDao;
use crate::state::Config;

/// Information about an item to be stored in the items list.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InitialItem {
    /// The name of the item.
    pub key: String,
    /// The value the item will have at instantiation time.
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// The name of the core contract.
    pub name: String,
    /// A description of the core contract.
    pub description: String,

    /// Instantiate information for the core contract's voting
    /// power module.
    pub voting_registry_module_instantiate_info: ModuleInstantiateInfo,
    /// Instantiate information for the core contract's
    /// proposal modules.
    // NOTE: the pre-propose-base package depends on it being the case
    // that the core module instantiates its proposal module.
    pub proposal_modules_instantiate_info: Vec<ModuleInstantiateInfo>,

    /// Initial information for arbitrary contract addresses to be
    /// added to the items map. The key is the name of the item in the
    /// items map. The value is an enum that either uses an existing
    /// address or instantiates a new contract.
    pub initial_items: Option<Vec<InitialItem>>,
    /// Implements the DAO Star standard: https://daostar.one/EIP
    pub dao_uri: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Callable by proposal modules. The DAO will execute the
    /// messages in the hook in order.
    ExecuteProposalHook { msgs: Vec<CosmosMsg<NeutronMsg>> },
    /// Pauses the DAO for a set duration.
    /// When paused the DAO is unable to execute proposals
    Pause { duration: Duration },
    /// Removes an item from the governance contract's item map.
    RemoveItem { key: String },
    /// Adds an item to the governance contract's item map. If the
    /// item already exists the existing value is overriden. If the
    /// item does not exist a new item is added.
    SetItem { key: String, addr: String },
    /// Callable by the core contract. Replaces the current
    /// governance contract config with the provided config.
    UpdateConfig { config: Config },
    /// Updates the governance contract's governance modules. Module
    /// instantiate info in `to_add` is used to create new modules and
    /// install them.
    UpdateProposalModules {
        // NOTE: the pre-propose-base package depends on it being the
        // case that the core module instantiates its proposal module.
        to_add: Vec<ModuleInstantiateInfo>,
        to_disable: Vec<String>,
    },
    /// Callable by the core contract. Replaces the current
    /// voting module with a new one instantiated by the governance
    /// contract.
    UpdateVotingModule { module: ModuleInstantiateInfo },
    /// Update the core module to add/remove SubDAOs and their charters
    UpdateSubDaos {
        to_add: Vec<SubDao>,
        to_remove: Vec<String>,
    },
}

#[voting_query]
#[info_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the contract's config. Returns Config.
    #[returns(Config)]
    Config {},
    /// Dumps all of the core contract's state in a single
    /// query. Useful for frontends as performance for queries is more
    /// limited by network times than compute times. Returns
    /// `DumpStateResponse`.
    #[returns(DumpStateResponse)]
    DumpState {},
    /// Gets the address associated with an item key.
    #[returns(GetItemResponse)]
    GetItem { key: String },
    /// Lists all of the items associted with the contract. For
    /// example, given the items `{ "group": "foo", "subdao": "bar"}`
    /// this query would return `[("group", "foo"), ("subdao",
    /// "bar")]`.
    #[returns(Vec<(String, String)>)]
    ListItems {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Gets all proposal modules associated with the
    /// contract. Returns Vec<ProposalModule>.
    #[returns(Vec<ProposalModule>)]
    ProposalModules {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Gets the active proposal modules associated with the
    /// contract. Returns Vec<ProposalModule>.
    #[returns(Vec<ProposalModule>)]
    ActiveProposalModules {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns information about if the contract is currently paused.
    #[returns(PauseInfoResponse)]
    PauseInfo {},
    /// Gets the contract's voting module. Returns Addr.
    #[returns(Addr)]
    VotingModule {},
    /// Returns all SubDAOs with their charters in a vec
    /// start_after is bound exclusive and asks for a string address
    #[returns(Vec<SubDao>)]
    ListSubDaos {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns the SubDAO for a specific address if it in the list
    #[returns(SubDao)]
    GetSubDao { address: String },
    /// Implements the DAO Star standard: https://daostar.one/EIP
    #[returns(Option<String>)]
    DaoURI {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
