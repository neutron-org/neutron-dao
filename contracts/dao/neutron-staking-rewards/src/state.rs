use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::{Item, Map};
use neutron_staking_rewards_common::error::ContractError;
use neutron_staking_rewards_common::types::{Config, State, UserInfo};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const USERS: Map<&Addr, UserInfo> = Map::new("users");

pub const PAUSED: Item<bool> = Item::new("paused");

pub fn assert_pause(storage: &dyn Storage) -> Result<(), ContractError> {
    if PAUSED.load(storage)? {
        return Err(ContractError::ContractPaused {});
    }

    Ok(())
}
