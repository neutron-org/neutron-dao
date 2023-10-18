use crate::error::ContractError;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    /// Distribution rate (0-1) which goes to distribution contract
    pub distribution_rate: Decimal,
    /// Address of distribution contract, which will receive funds defined but distribution_rate %
    pub distribution_contract: Addr,
    /// Address of treasury contract, which will receive funds defined by 100-distribution_rate %
    pub treasury_contract: Addr,
    /// Minimum period between distribution calls
    pub min_period: u64,
    pub denom: String,

    /// Address of the main DAO contract
    pub main_dao_address: Addr,

    /// Address of the security DAO contract
    pub security_dao_address: Addr,

    // Denominator used in the vesting release function
    pub vesting_denominator: u128,
}

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        if (self.distribution_rate > Decimal::one()) || (self.distribution_rate < Decimal::zero()) {
            return Err(ContractError::InvalidDistributionRate {});
        }

        if self.vesting_denominator == 0 {
            return Err(ContractError::InvalidVestingDenominator {});
        }

        if self.min_period == 0 {
            return Err(ContractError::InvalidMinPeriod {});
        }

        Ok(())
    }
}

pub const TOTAL_DISTRIBUTED: Item<Uint128> = Item::new("total_distributed");
pub const TOTAL_RESERVED: Item<Uint128> = Item::new("total_reserved");

pub const LAST_BURNED_COINS_AMOUNT: Item<Uint128> = Item::new("last_burned_coins_amount");
pub const LAST_DISTRIBUTION_TIME: Item<u64> = Item::new("last_grab_time");

pub const CONFIG: Item<Config> = Item::new("config");

/// The height the contract is paused until. If it's None, the contract is not paused.
pub const PAUSED_UNTIL: Item<Option<u64>> = Item::new("paused_until");

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::error::ContractError;
    use cosmwasm_std::Addr;
    use cosmwasm_std::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_config_validate() {
        let cfg_ok = Config {
            distribution_rate: Decimal::from_str("0.11").unwrap(),
            distribution_contract: Addr::unchecked("owner"),
            treasury_contract: Addr::unchecked("owner"),
            min_period: 3600,
            denom: String::from("untrn"),
            main_dao_address: Addr::unchecked("owner"),
            security_dao_address: Addr::unchecked("owner"),
            vesting_denominator: 100_000_000_000u128,
        };
        assert_eq!(cfg_ok.validate(), Ok(()));

        let cfg_invalid_distr = Config {
            distribution_rate: Decimal::from_str("111.11").unwrap(),
            distribution_contract: Addr::unchecked("owner"),
            treasury_contract: Addr::unchecked("owner"),
            min_period: 3600,
            denom: String::from("untrn"),
            main_dao_address: Addr::unchecked("owner"),
            security_dao_address: Addr::unchecked("owner"),
            vesting_denominator: 100_000_000_000u128,
        };
        assert_eq!(
            cfg_invalid_distr.validate(),
            Err(ContractError::InvalidDistributionRate {})
        );

        let cfg_invalid_vesting_denom = Config {
            distribution_rate: Decimal::from_str("0.11").unwrap(),
            distribution_contract: Addr::unchecked("owner"),
            treasury_contract: Addr::unchecked("owner"),
            min_period: 3600,
            denom: String::from("untrn"),
            main_dao_address: Addr::unchecked("owner"),
            security_dao_address: Addr::unchecked("owner"),
            vesting_denominator: 0u128,
        };
        assert_eq!(
            cfg_invalid_vesting_denom.validate(),
            Err(ContractError::InvalidVestingDenominator {})
        );

        let cfg_invalid_min_period = Config {
            distribution_rate: Decimal::from_str("0.11").unwrap(),
            distribution_contract: Addr::unchecked("owner"),
            treasury_contract: Addr::unchecked("owner"),
            min_period: 0,
            denom: String::from("untrn"),
            main_dao_address: Addr::unchecked("owner"),
            security_dao_address: Addr::unchecked("owner"),
            vesting_denominator: 100_000_000_000u128,
        };
        assert_eq!(
            cfg_invalid_min_period.validate(),
            Err(ContractError::InvalidMinPeriod {})
        );
    }
}
