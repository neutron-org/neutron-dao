use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::{Item, Map};

/// Configuration.
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub dao_address: Addr,
    pub staking_info_proxy: Addr,
    pub annual_reward_rate_bps: u64,
    pub blocks_per_year: u64,
    pub staking_denom: String,
}

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.staking_denom.len() < 1 {
            return Err(ContractError::EmptyStakeDenom {});
        }

        if self.blocks_per_year < 1 {
            return Err(ContractError::ZeroBlocksPerYear {});
        }

        Ok(())
    }
}

/// Frequently updated reward-related data.
#[cw_serde]
pub struct State {
    pub global_reward_index: Decimal,
    pub last_global_update_block: u64,
}

/// Per-user info about stake, reward index, and accrued rewards.
#[cw_serde]
pub struct UserInfo {
    pub user_reward_index: Decimal,
    pub stake: Coin,
    pub pending_rewards: Coin,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const USERS: Map<&Addr, UserInfo> = Map::new("users");
