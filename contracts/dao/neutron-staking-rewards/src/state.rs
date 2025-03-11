use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use neutron_staking_rewards_common::types::{Config, State, UserInfo};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const USERS: Map<&Addr, UserInfo> = Map::new("users");
