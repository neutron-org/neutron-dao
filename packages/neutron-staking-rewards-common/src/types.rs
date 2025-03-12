use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal};

/// Configuration.
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub dao_address: Addr,
    pub staking_info_proxy: Addr,
    pub annual_reward_rate_bps: u64,
    pub blocks_per_year: u64,
    pub staking_denom: String,
    // an address that is allowed to pause/unpause the contract
    pub security_address: Addr,
}

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.staking_denom.is_empty() {
            return Err(ContractError::EmptyStakeDenom {});
        }

        if self.blocks_per_year < 1 {
            return Err(ContractError::ZeroBlocksPerYear {});
        }

        Ok(())
    }
}

type SlashingEvent = (GlobalRewardIndex, u64);
type GlobalRewardIndex = Decimal;

/// Frequently updated reward-related data.
#[cw_serde]
pub struct State {
    pub global_reward_index: GlobalRewardIndex,
    pub global_update_height: u64,
    pub slashing_events: Vec<SlashingEvent>,
}

impl State {
    pub fn load_unprocessed_slashing_events(&self, from_height: u64) -> Vec<SlashingEvent> {
        let events = self
            .slashing_events
            .iter()
            .skip_while(|&&(_, event_height)| event_height < from_height)
            .cloned()
            .collect();
        events
    }
}

/// Per-user info about stake, reward index, and accrued rewards.
#[cw_serde]
pub struct UserInfo {
    pub stake: Coin,
    pub user_reward_index: Decimal,
    pub last_update_block: u64,
    pub pending_rewards: Coin,
}
