use cosmwasm_std::{OverflowError, StdError};
use exec_control::pause::PauseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    PauseError(#[from] PauseError),

    #[error("Unauthorized.")]
    Unauthorized {},

    #[error("No funds sent.")]
    NoFundsSent {},

    #[error("No shares set.")]
    NoSharesSent {},

    #[error("No pending distribution.")]
    NoPendingDistribution {},
}

impl From<OverflowError> for ContractError {
    fn from(o: OverflowError) -> Self {
        StdError::from(o).into()
    }
}
