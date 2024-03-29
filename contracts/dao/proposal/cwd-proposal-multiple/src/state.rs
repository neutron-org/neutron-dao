use crate::proposal::MultipleChoiceProposal;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;
use cwd_hooks::Hooks;
use cwd_voting::{
    multiple_choice::{MultipleChoiceVote, VotingStrategy},
    pre_propose::ProposalCreationPolicy,
};

/// The proposal module's configuration.
#[cw_serde]
pub struct Config {
    /// The threshold a proposal must reach to complete.
    pub voting_strategy: VotingStrategy,
    /// The minimum amount of time a proposal must be open before
    /// passing. A proposal may fail before this amount of time has
    /// elapsed, but it will not pass. This can be useful for
    /// preventing governance attacks wherein an attacker aquires a
    /// large number of tokens and forces a proposal through.
    pub min_voting_period: Option<Duration>,
    /// The default maximum amount of time a proposal may be voted on
    /// before expiring.
    pub max_voting_period: Duration,
    /// If set to true only members may execute passed
    /// proposals. Otherwise, any address may execute a passed
    /// proposal.
    pub only_members_execute: bool,
    /// Allows changing votes before the proposal expires. If this is
    /// enabled proposals will not be able to complete early as final
    /// vote information is not known until the time of proposal
    /// expiration.
    pub allow_revoting: bool,
    /// The address of the DAO that this governance module is
    /// associated with.
    pub dao: Addr,
    /// If set to true proposals will be closed if their execution
    /// fails. Otherwise, proposals will remain open after execution
    /// failure. For example, with this enabled a proposal to send 5
    /// tokens out of a DAO's reserve with 4 tokens would be closed when
    /// it is executed. With this disabled, that same proposal would
    /// remain open until the DAO's reserve was large enough for it to be
    /// executed.
    pub close_proposal_on_execution_failure: bool,
}

// we cast a ballot with our chosen vote and a given weight
// stored under the key that voted
#[cw_serde]
pub struct Ballot {
    /// The amount of voting power behind the vote.
    pub power: Uint128,
    /// The position.
    pub vote: MultipleChoiceVote,
}

/// The current top level config for the module.
pub const CONFIG: Item<Config> = Item::new("config");
pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");
pub const PROPOSALS: Map<u64, MultipleChoiceProposal> = Map::new("proposals");
pub const BALLOTS: Map<(u64, Addr), Ballot> = Map::new("ballots");
/// Consumers of proposal state change hooks.
pub const PROPOSAL_HOOKS: Hooks = Hooks::new("proposal_hooks");
/// Consumers of vote hooks.
pub const VOTE_HOOKS: Hooks = Hooks::new("vote_hooks");
/// The address of the pre-propose module associated with this
/// proposal module (if any).
pub const CREATION_POLICY: Item<ProposalCreationPolicy> = Item::new("creation_policy");
/// Execution errors for proposals that do not close on failure (Config.close_proposal_on_execution_failure set to false)
pub const PROPOSAL_EXECUTION_ERRORS: Map<u64, String> = Map::new("proposal_execution_errors");
