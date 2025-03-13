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
}

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.staking_denom.is_empty() {
            return Err(ContractError::EmptyStakeDenom {});
        }

        if self.blocks_per_year < 1 {
            return Err(ContractError::ZeroBlocksPerYear {});
        }

        if self.annual_reward_rate_bps > 10_000 {
            return Err(ContractError::InvalidBPS {bps: self.annual_reward_rate_bps});
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        struct TestCase {
            name: &'static str,
            config: Config,
            expected_result: Result<(), ContractError>,
        }

        let test_cases = vec![
            TestCase {
                name: "valid configuration",
                config: Config {
                    owner: Addr::unchecked("owner"),
                    dao_address: Addr::unchecked("dao"),
                    staking_info_proxy: Addr::unchecked("proxy"),
                    annual_reward_rate_bps: 500, // 5%
                    blocks_per_year: 5_256_000,  // Approximately 6 seconds per block
                    staking_denom: "ustake".to_string(),
                },
                expected_result: Ok(()),
            },
            TestCase {
                name: "empty staking denomination",
                config: Config {
                    owner: Addr::unchecked("owner"),
                    dao_address: Addr::unchecked("dao"),
                    staking_info_proxy: Addr::unchecked("proxy"),
                    annual_reward_rate_bps: 500,
                    blocks_per_year: 5_256_000,
                    staking_denom: "".to_string(),
                },
                expected_result: Err(ContractError::EmptyStakeDenom {}),
            },
            TestCase {
                name: "zero blocks per year",
                config: Config {
                    owner: Addr::unchecked("owner"),
                    dao_address: Addr::unchecked("dao"),
                    staking_info_proxy: Addr::unchecked("proxy"),
                    annual_reward_rate_bps: 500,
                    blocks_per_year: 0,
                    staking_denom: "ustake".to_string(),
                },
                expected_result: Err(ContractError::ZeroBlocksPerYear {}),
            },
            TestCase {
                name: "invalid BPS (>10,000)",
                config: Config {
                    owner: Addr::unchecked("owner"),
                    dao_address: Addr::unchecked("dao"),
                    staking_info_proxy: Addr::unchecked("proxy"),
                    annual_reward_rate_bps: 12_000,
                    blocks_per_year: 5_256_000,
                    staking_denom: "ustake".to_string(),
                },
                expected_result: Err(ContractError::InvalidBPS { bps: 12_000 }),
            },
            TestCase {
                name: "maximum valid BPS (10,000)",
                config: Config {
                    owner: Addr::unchecked("owner"),
                    dao_address: Addr::unchecked("dao"),
                    staking_info_proxy: Addr::unchecked("proxy"),
                    annual_reward_rate_bps: 10_000,
                    blocks_per_year: 5_256_000,
                    staking_denom: "ustake".to_string(),
                },
                expected_result: Ok(()),
            },
            TestCase {
                name: "minimum valid blocks per year (1)",
                config: Config {
                    owner: Addr::unchecked("owner"),
                    dao_address: Addr::unchecked("dao"),
                    staking_info_proxy: Addr::unchecked("proxy"),
                    annual_reward_rate_bps: 500,
                    blocks_per_year: 1,
                    staking_denom: "ustake".to_string(),
                },
                expected_result: Ok(()),
            },
        ];

        for tc in test_cases {
            let result = tc.config.validate();
            
            match (&result, &tc.expected_result) {
                (Ok(_), Ok(_)) => {
                    // Both are Ok, test passes
                },
                (Err(e1), Err(e2)) => {
                    // Compare error variants
                    assert_eq!(
                        format!("{:?}", e1),
                        format!("{:?}", e2),
                        "Test case '{}' failed: expected {:?}, got {:?}",
                        tc.name,
                        tc.expected_result,
                        result
                    );
                },
                _ => {
                    panic!(
                        "Test case '{}' failed: expected {:?}, got {:?}",
                        tc.name,
                        tc.expected_result,
                        result
                    );
                }
            }
        }
    }
}