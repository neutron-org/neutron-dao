use cosmwasm_std::{Addr};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub owner: Addr,
    pub true_admin: Addr,
    pub eol: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
