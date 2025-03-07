use cosmwasm_std::Addr;
use cw_storage_plus::{Item, SnapshotMap, Strategy};
use neutron_staking_tracker_common::types::{Config, Delegation, Validator};

/// Storage mapping for all validators, indexed by **operator address (`valoper`)**.
///
/// This stores validator information under their **operator address**.
/// - **Key:** `&Addr` → Validator's **operator address** (`valoper`).
/// - **Value:** `Validator` struct containing all validator details.
///
/// We use `SnapshotMap` to enable querying historical validator states at any height.
pub const VALIDATORS: SnapshotMap<&Addr, Validator> = SnapshotMap::new(
    "validators",
    "validators__checkpoints",
    "validators__changelog",
    Strategy::EveryBlock,
);

/// Storage mapping for delegations, indexed by **(delegator, validator operator address)**.
///
/// This stores delegation details for each user that stakes with a validator.
/// - **Key:** `(&Addr, &Addr)` → **(delegator address, validator operator address (`valoper`))**.
/// - **Value:** `Delegation` struct containing delegation details.
///
/// We use `SnapshotMap` to allow tracking delegation history over time.
pub const DELEGATIONS: SnapshotMap<(&Addr, &Addr), Delegation> = SnapshotMap::new(
    "delegations",
    "delegations__checkpoints",
    "delegations__changelog",
    Strategy::EveryBlock,
);

/// Stores the core **configuration** of the contract.
///
/// Contains metadata such as the contract's **name, description, owner, and token denom**.
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the **DAO address** responsible for managing governance decisions.
pub const DAO: Item<Addr> = Item::new("dao");
