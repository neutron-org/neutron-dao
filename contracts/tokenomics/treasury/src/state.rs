use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub denom: String,
    /// The address of the main DAO. It's capable of pausing and unpausing the contract
    pub main_dao_address: Addr,
    /// The address of the DAO guardian. The security DAO is capable only of pausing the contract.
    pub security_dao_address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

/// The height the contract is paused until. If it's None, the contract is not paused.
pub const PAUSED_UNTIL: Item<Option<u64>> = Item::new("paused_until");
