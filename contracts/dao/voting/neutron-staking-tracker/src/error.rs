use cosmwasm_std::{
    CheckedFromRatioError, ConversionOverflowError, Decimal256RangeExceeded, DecimalRangeExceeded,
    DivideByZeroError, OverflowError, StdError,
};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

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

    #[error("Configuration denom cannot be empty.")]
    DenomIsEmpty {},

    #[error("Unauthorized action.")]
    Unauthorized {},

    #[error("Validator not found: {address}")]
    ValidatorNotFound { address: String },

    #[error("Validator already exists: {address}")]
    ValidatorAlreadyExists { address: String },

    #[error("Validator is already active: {address}")]
    ValidatorAlreadyActive { address: String },

    #[error("Validator is not active: {address}")]
    ValidatorNotActive { address: String },

    #[error("Validator is not bonded: {validator}")]
    ValidatorNotBonded { validator: String },

    #[error("Delegation not found for delegator: {delegator}, validator: {validator}")]
    DelegationNotFound {
        delegator: String,
        validator: String,
    },

    #[error("Bonding operations are disabled for this contract.")]
    BondingDisabled {},

    #[error("Direct unbonding operations are disabled for this contract.")]
    DirectUnbondingDisabled {},

    #[error("Insufficient funds for the requested operation.")]
    InsufficientFunds {},

    #[error("Cannot slash the specified validator: {validator}")]
    ValidatorSlashingError { validator: String },

    #[error("Validator data is missing in the query response: {address}")]
    NoPubKey { address: String },

    #[error("Invalid token data for validator: {address}")]
    InvalidTokenData { address: String },

    #[error(transparent)]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

    #[error(transparent)]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("Failed to query validator: {address}")]
    ValidatorQueryFailed { address: String },

    #[error("Failed to calculate consensus key")]
    InvalidConsensusKey,

    #[error("Failed to calculate consensus key: {validator}")]
    DelegationQueryFailed { validator: String },

    #[error("Failed to get delegation balance : {delegator} {validator}")]
    DelegationBalanceNotFound {
        delegator: String,
        validator: String,
    },

    #[error("Generic overflow error occurred.")]
    OverflowError(#[from] OverflowError),

    #[error("Invalid shares: {shares_str}. Err: {err}")]
    InvalidSharesFormat { shares_str: String, err: String },
}
