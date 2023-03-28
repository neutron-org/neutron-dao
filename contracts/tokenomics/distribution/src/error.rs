use cosmwasm_std::{DivideByZeroError, OverflowError, StdError};
use exec_control::pause::PauseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    PauseError(#[from] PauseError),

    #[error(transparent)]
    OverflowError(#[from] OverflowError),

    #[error(transparent)]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("Unauthorized.")]
    Unauthorized {},

    #[error("No funds sent.")]
    NoFundsSent {},

    #[error("No shares sent.")]
    NoSharesSent {},

    #[error("No pending distribution.")]
    NoPendingDistribution {},
}
