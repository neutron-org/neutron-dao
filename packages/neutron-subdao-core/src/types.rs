use cosmwasm_std::{Addr, Uint128};
use cw2::ContractVersion;
use exec_control::pause::PauseInfoResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Relevant state for the governance module. Returned by the
/// `DumpState` query.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DumpStateResponse {
    /// The governance contract's config.
    pub config: Config,
    /// Whether the contract is currently paused.
    pub pause_info: PauseInfoResponse,
    /// The governance contract's version.
    pub version: ContractVersion,
    /// The governance modules associated with the governance
    /// contract.
    pub proposal_modules: Vec<ProposalModule>,
    /// The voting module associated with the governance contract.
    pub voting_module: Addr,
    /// The number of active proposal modules.
    pub active_proposal_module_count: u32,
    /// The total number of proposal modules.
    pub total_proposal_module_count: u32,
}

/// Returned by the `GetItem` query.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct GetItemResponse {
    /// `None` if no item with the provided key was found, `Some`
    /// otherwise.
    pub item: Option<String>,
}

/// Returned by the `Cw20Balances` query.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Cw20BalanceResponse {
    /// The address of the token.
    pub addr: Addr,
    /// The contract's balance.
    pub balance: Uint128,
}

/// Returned by the `AdminNomination` query.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AdminNominationResponse {
    /// The currently nominated admin or None if no nomination is
    /// pending.
    pub nomination: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubDao {
    /// The contract address of the SubDAO
    pub addr: String,
    /// The purpose/constitution for the SubDAO
    pub charter: Option<String>,
}

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
    pub security_dao: Addr,
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
