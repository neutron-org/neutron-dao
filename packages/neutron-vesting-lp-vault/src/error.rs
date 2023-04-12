use cosmwasm_std::{ConversionOverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Bonding is not available for this contract")]
    BondingDisabled {},

    #[error("Direct unbonding is not available for this contract")]
    DirectUnbondingDisabled {},

    #[error("Only owner can change owner")]
    OnlyOwnerCanChangeOwner {},

    #[error("Only owner can change vesting LP contract")]
    OnlyOwnerCanChangeVestingLpContract {},

    #[error("config name cannot be empty.")]
    NameIsEmpty {},

    #[error("config description cannot be empty.")]
    DescriptionIsEmpty {},
}

pub type ContractResult<T> = Result<T, ContractError>;
