use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
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

/// Map to store the amount of funds that are pending distribution to a given address
pub const PENDING_DISTRIBUTION: Map<Addr, Uint128> = Map::new("pending_distribution");

/// Map to store the amount of shares that a given address has
pub const SHARES: Map<Addr, Uint128> = Map::new("shares");

/// Stores contract's configuration
pub const CONFIG: Item<Config> = Item::new("config");

/// Counts times `Fund` has been called successfully.
/// Used to determine what address to fund non-dividable remainder after weight distribution.
pub const FUND_COUNTER: Item<u64> = Item::new("fund_counter");

/// The height the contract is paused until. If it's None, the contract is not paused.
pub const PAUSED_UNTIL: Item<Option<u64>> = Item::new("paused_until");
