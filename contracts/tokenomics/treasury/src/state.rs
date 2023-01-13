use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    /// Distribution rate (0-1) which goes to distribution contract
    pub distribution_rate: Decimal,
    /// Address of distribution contract, which will receive funds defined but distribution_rate %
    pub distribution_contract: Addr,
    /// Address of reserve contract, which will receive funds defined by 100-distribution_rate %
    pub reserve_contract: Addr,
    /// Minimum period between distribution calls
    pub min_period: u64,
    pub denom: String,

    /// Address of the main DAO contract
    pub main_dao_address: Addr,

    /// Address of the security DAO contract
    pub security_dao_address: Addr,
}

pub const TOTAL_RECEIVED: Item<Uint128> = Item::new("total_received");
pub const TOTAL_DISTRIBUTED: Item<Uint128> = Item::new("total_distributed");
pub const TOTAL_RESERVED: Item<Uint128> = Item::new("total_reserved");

pub const LAST_DISTRIBUTION_TIME: Item<u64> = Item::new("last_grab_time");

pub const CONFIG: Item<Config> = Item::new("config");

/// The height the subDAO is paused until. If it's None, the subDAO is not paused.
pub const PAUSED_UNTIL: Item<Option<u64>> = Item::new("paused_until");
