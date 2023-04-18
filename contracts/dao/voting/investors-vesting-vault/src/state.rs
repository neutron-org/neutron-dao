use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub vesting_contract_address: Addr,
    pub description: String,
    pub owner: Addr,
    pub manager: Option<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");
pub const DESCRIPTION: Item<String> = Item::new("description");
