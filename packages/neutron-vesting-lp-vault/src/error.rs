use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Bonding is not available for this contract")]
    BondingDisabled {},

    #[error("Direct unbonding is not available for this contract")]
    DirectUnbondingDisabled {},

    #[error("config name cannot be empty.")]
    NameIsEmpty {},

    #[error("config description cannot be empty.")]
    DescriptionIsEmpty {},

    #[error("{0}")]
    OverflowError(#[from] OverflowError),
}

pub type ContractResult<T> = Result<T, ContractError>;
