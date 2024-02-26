use crate::msg::Strategy;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

/// Defines a mapping from an address to a strategy associated with the address.
pub const STRATEGIES: Map<Addr, Strategy> = Map::new("chain-manager-strategies");
