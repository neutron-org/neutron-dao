use cosmwasm_std::{
    CheckedFromRatioError, ConversionOverflowError, Decimal256RangeExceeded, DecimalRangeExceeded,
    DivideByZeroError, OverflowError, StdError,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Math error occurred: {error}")]
    MathError { error: String },

    #[error(transparent)]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error(transparent)]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("Configuration name cannot be empty.")]
    NameIsEmpty {},

    #[error("Configuration description cannot be empty.")]
    DescriptionIsEmpty {},

    #[error("Unauthorized action.")]
    Unauthorized {},

    #[error("Validator not found in staking module: {address}")]
    ValidatorNotFound { address: String },

    #[error(transparent)]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

    #[error(transparent)]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("Generic overflow error occurred.")]
    OverflowError(#[from] OverflowError),

    #[error("Invalid shares: {shares_str}. Err: {err}")]
    InvalidSharesFormat { shares_str: String, err: String },

    #[error("Unsupported hook: {hook}")]
    UnsupportedHook { hook: String },

    #[error("ValidatorAlreadyBonded: {address}")]
    ValidatorAlreadyBonded { address: String },
}
