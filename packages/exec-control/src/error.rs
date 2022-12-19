use crate::types::MAX_PAUSE_DURATION;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ExecControlError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized.")]
    Unauthorized {},

    #[error(
        "Pause duration is too big: it's only possible to pause the contract for {} blocks.",
        MAX_PAUSE_DURATION
    )]
    PauseDurationTooBig {},

    #[error("Contract execution is paused.")]
    Paused {},
}
