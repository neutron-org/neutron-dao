use cosmwasm_std::{OverflowError, StdError};
use exec_control::pause::PauseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    PauseError(#[from] PauseError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No funds to distribute")]
    NoFundsToDistribute {},

    #[error("distribution_rate must be between 0 and 1")]
    InvalidDistributionRate {},

    #[error("vesting_denominator must be more than zero")]
    InvalidVestingDenominator {},

    #[error("Too soon to distribute")]
    TooSoonToDistribute {},

    #[error("no coins were burned, nothing to distribute")]
    NoBurnedCoins {},

    #[error("Overflow")]
    OverflowError(#[from] OverflowError),
}
