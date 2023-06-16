use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use neutron_lockdrop_vault::types::{Config, OldConfig};

pub const CONFIG: Item<Config> = Item::new("config");
pub const OLD_CONFIG: Item<OldConfig> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");
