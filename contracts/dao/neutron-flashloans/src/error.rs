use cosmwasm_std::{CheckedMultiplyRatioError, OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    // This error is returned if you requested a specific coin several times.
    #[error("Duplicate denoms requested")]
    DuplicateDenoms {},

    // This error is returned if you requested zero of a specific coin.
    #[error("Zero amount of {denom} requested")]
    ZeroRequested { denom: String },

    // This error is returned when you try to get a flashloan when you already
    // have one.
    #[error("A flashloan is already active in this transaction")]
    FlashloanAlreadyActive {},

    // This error is returned if the SOURCE doesn't have the requested amount of
    // one of the requested assets.
    #[error("Source doesn't have enough {denom}")]
    InsufficientFunds { denom: String },

    // This error is returned if the borrower did not return **exactly** the loan plus the fee
    // to the source.
    #[error("Borrower did not return exactly (loan + fee)")]
    IncorrectPayback {},

    // This error is returned if the contract received an unknown reply ID.
    #[error("Unknown reply id: {id}")]
    UnknownReplyID { id: u64 },

    // This error is returned when we can't find the active loan information in our
    // reply handlers. It's not supposed to occur at all.
    #[error("Unexpected state: can't find active loan information")]
    UnexpectedState {},

    #[error("CheckedMultiplyRatioError error: {0}")]
    CheckedMultiplyRatioError(#[from] CheckedMultiplyRatioError),

    #[error("OverflowError error: {0}")]
    OverflowError(#[from] OverflowError),
}
