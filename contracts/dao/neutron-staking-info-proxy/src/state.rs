use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use neutron_staking_info_proxy_common::types::Config;

/// List of providers for querying staking information.
/// A provider is a contract that supplies stake information updates.
pub const PROVIDERS: Map<Addr, ()> = Map::new("providers");

/// Contract's configuration parameters.
pub const CONFIG: Item<Config> = Item::new("config");
