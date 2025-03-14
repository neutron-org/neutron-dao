use cosmwasm_std::Addr;
use cw_storage_plus::{Item, SnapshotItem, SnapshotMap, Strategy};
use neutron_staking_tracker_common::types::{Config, Delegation, Validator};

/// Storage mapping for all validators, indexed by the **operator address (`valoper`)**.
///
/// Stores validator information under their **operator address**.
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

/// Stores the list of bonded validators.
/// The point of storing `BONDED_VALIDATORS_SET` is to avoid (potentially big) iteration over all validators.
/// This can happen since SnapshotMap cannot iterate over previous heights,
/// and so without ability to list bonded validators at any height,
/// full `VALIDATORS` iteration is necessary to calculate stake at height.
///
/// -- **Value:** `Vec<String>` contains list of validator addresses (`valoper`)
///
/// We use `SnapshotItem` to allow tracking bonded validators set history over time.
pub const BONDED_VALIDATORS_SET: SnapshotItem<Vec<String>> = SnapshotItem::new(
    "bonded_validators_set",
    "bonded_validators_set__checkpoints",
    "bonded_validators_set__changelog",
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
