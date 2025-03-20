use crate::{ContractError, ContractResult};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, SnapshotItem, Strategy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    /// Name of the contract.
    pub name: String,
    /// Description of the contract.
    pub description: String,
    /// Address of staking tracker contract.
    /// Used for querying user's and total stake at height.
    pub staking_tracker_contract_address: Addr,
    /// Contract's owner that can update config and manage blacklist
    pub owner: Addr,
}

impl Config {
    /// checks whether the config fields are valid.
    pub fn validate(&self) -> ContractResult<()> {
        if self.name.is_empty() {
            return Err(ContractError::NameIsEmpty {});
        }
        if self.description.is_empty() {
            return Err(ContractError::DescriptionIsEmpty {});
        }
        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");

/// If an address is blacklisted, its stake is **excluded** from governance and voting power calculations.
///
/// - **Value:** `Vec<Addr>` -> The blacklisted wallet addresses.
///
/// We use `SnapshotItem` to enable querying blacklisted addresses at any height.
pub const BLACKLISTED_ADDRESSES: SnapshotItem<Vec<Addr>> = SnapshotItem::new(
    "blacklisted_addresses",
    "blacklisted_addresses__checkpoints",
    "blacklisted_addresses__changelog",
    Strategy::EveryBlock,
);
