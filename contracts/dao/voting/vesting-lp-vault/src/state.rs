use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use neutron_vesting_lp_vault::types::Config;

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");
