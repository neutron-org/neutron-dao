use cosmwasm_std::Addr;
use cw_storage_plus::{Item, SnapshotMap, Strategy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub owner: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");

/// Snapshots of vault states throughout the chain life.
pub const VAULT_STATES: SnapshotMap<Addr, VotingVaultState> = SnapshotMap::new(
    "voting_vault_state",
    "voting_vault_state__checkpoints",
    "voting_vault_state__changelog",
    Strategy::EveryBlock,
);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum VotingVaultState {
    /// Voting vault is active means that it's considered in voting power queries to the
    /// Neutron voting registry.
    Active,
    /// Voting vault is inactive means that it's not considered in voting power queries to the
    /// Neutron voting registry.
    Inactive,
}
