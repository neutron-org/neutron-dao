use cosmwasm_std::StdError;
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

    #[error("Insufficient funds.")]
    InsufficientFunds {},
}
