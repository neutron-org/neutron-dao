use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub credits_contract_address: Addr,
    pub description: String,
    pub owner: Addr,
}

#[cw_serde]
pub struct TotalSupplyResponse {
    // Total supply of ucNTRNs for specified block height
    pub total_supply: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");
pub const DESCRIPTION: Item<String> = Item::new("description");
