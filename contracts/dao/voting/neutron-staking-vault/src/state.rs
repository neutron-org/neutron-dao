use crate::error::ContractError;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map, SnapshotMap, Strategy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration settings for the smart contract.
///
/// This struct holds key details about the vault, including:
/// - `name`: The name of the vault.
/// - `description`: A short text description of the vault.
/// - `owner`: The address of the vault owner/admin.
/// - `denom`: The token denomination used for delegations and governance.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub owner: Addr,
    pub denom: String,
}

impl Config {
    /// Validates whether the configuration parameters are correctly set.
    ///
    /// - Ensures that `name`, `description`, and `denom` are not empty.
    /// - If any field is invalid, returns an appropriate `ContractError`.
    ///
    /// Returns:
    /// - `Ok(())` if all fields are valid.
    /// - `Err(ContractError)` if validation fails.
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.name.is_empty() {
            return Err(ContractError::NameIsEmpty {});
        };
        if self.description.is_empty() {
            return Err(ContractError::DescriptionIsEmpty {});
        };
        if self.denom.is_empty() {
            return Err(ContractError::DenomIsEmpty {});
        };
        Ok(())
    }
}

/// Represents a validator in the staking system.
///
/// A validator is responsible for securing the network and participating in consensus.
/// Each validator has:
/// - `cons_address`: The **consensus address** (`valcons`), used for signing blocks.
/// - `oper_address`: The **operator address** (`valoper`), used for staking/delegation.
/// - `bonded`: Whether the validator is bonded (actively participating in consensus).
/// - `total_tokens`: Total staked tokens delegated to this validator.
/// - `total_shares`: Total delegation shares representing ownership over the staked tokens.
/// - `active`: Whether the validator is active in the network.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Validator {
    pub cons_address: Addr,
    pub oper_address: Addr,
    pub bonded: bool,
    pub total_tokens: Uint128,
    pub total_shares: Uint128,
    pub active: bool,
}

/// Represents a delegation made by a user to a validator.
///
/// A delegation means that a **delegator** (user) has assigned their stake to a **validator**.
/// - `delegator_address`: The user's wallet address that owns the stake.
/// - `validator_address`: The operator address (`valoper`) of the validator receiving the delegation.
/// - `shares`: The amount of **delegation shares** received in exchange for staked tokens.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Delegation {
    pub delegator_address: Addr,
    pub validator_address: Addr,
    pub shares: Uint128,
}

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

/// If an address is blacklisted, its stake is **excluded** from governance and voting power calculations.
/// - **Key:** `Addr` → The blacklisted wallet address.
/// - **Value:** `bool` → `true` if blacklisted.
pub const BLACKLISTED_ADDRESSES: Map<Addr, bool> = Map::new("blacklisted_addresses");

/// Stores the core **configuration** of the contract.
///
/// Contains metadata such as the contract's **name, description, owner, and token denom**.
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the **DAO address** responsible for managing governance decisions.
pub const DAO: Item<Addr> = Item::new("dao");

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::error::ContractError;
    use cosmwasm_std::{Addr, Storage};

    /// Tests the validation logic for the `Config` struct.
    ///
    /// Ensures that empty fields are properly rejected with the correct `ContractError`.
    #[test]
    fn test_config_validate() {
        let cfg_ok = Config {
            name: String::from("name"),
            description: String::from("description"),
            owner: Addr::unchecked("owner"),
            denom: String::from("denom"),
        };
        assert_eq!(cfg_ok.validate(), Ok(()));

        let cfg_empty_name = Config {
            name: String::from(""),
            description: String::from("description"),
            owner: Addr::unchecked("owner"),
            denom: String::from("denom"),
        };
        assert_eq!(
            cfg_empty_name.validate(),
            Err(ContractError::NameIsEmpty {})
        );

        let cfg_empty_description = Config {
            name: String::from("name"),
            description: String::from(""),
            owner: Addr::unchecked("owner"),
            denom: String::from("denom"),
        };
        assert_eq!(
            cfg_empty_description.validate(),
            Err(ContractError::DescriptionIsEmpty {})
        );

        let cfg_empty_denom = Config {
            name: String::from("name"),
            description: String::from("description"),
            owner: Addr::unchecked("owner"),
            denom: String::from(""),
        };
        assert_eq!(
            cfg_empty_denom.validate(),
            Err(ContractError::DenomIsEmpty {})
        );
    }

}
