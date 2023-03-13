use cosmwasm_std::Addr;
use cw_storage_plus::{Item};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub main_dao: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config_overrule");
