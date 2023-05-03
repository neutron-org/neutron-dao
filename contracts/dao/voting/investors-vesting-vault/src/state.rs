use crate::{ContractError, ContractResult};
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub vesting_contract_address: Addr,
    pub description: String,
    pub owner: Addr,
    pub name: String,
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
