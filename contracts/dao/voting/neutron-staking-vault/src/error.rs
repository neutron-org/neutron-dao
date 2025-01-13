use cosmwasm_std::{OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("{0}")]
    MathError { error: String },

    #[error("Config name cannot be empty.")]
    NameIsEmpty {},

    #[error("Config description cannot be empty.")]
    DescriptionIsEmpty {},

    #[error("Config denom cannot be empty.")]
    DenomIsEmpty {},

    #[error("Unauthorized")]
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

    #[error("Delegation not found for delegator: {delegator} and validator: {validator}")]
    DelegationNotFound {
        delegator: String,
        validator: String,
    },

    #[error("Bonding is not available for this contract")]
    BondingDisabled {},

    #[error("Direct unbonding is not available for this contract")]
    DirectUnbondingDisabled {},

    #[error("Insufficient funds for operation")]
    InsufficientFunds {},

    #[error("Cannot slash validator: {validator}")]
    ValidatorSlashingError { validator: String },
}
