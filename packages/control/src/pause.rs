use cosmwasm_std::Addr;
use thiserror::Error;

/// Approximately one week given block time = 2sec.
pub const MAX_PAUSE_DURATION: u64 = 302400;

/// checks whether the sender is either the admin or the guardian if any.
pub fn validate_sender(
    sender: Addr,
    admin: Addr,
    guardian: Option<Addr>,
) -> Result<(), PauseError> {
    let authorized = match guardian {
        Some(g) => sender == admin || sender == g,
        None => sender == admin,
    };
    if !authorized {
        return Err(PauseError::Unauthorized {});
    }
    Ok(())
}

/// checks whether the duration is not greater than MAX_PAUSE_DURATION.
pub fn validate_duration(duration: u64) -> Result<(), PauseError> {
    if duration.gt(&MAX_PAUSE_DURATION) {
        return Err(PauseError::PauseDurationTooBig {});
    }
    Ok(())
}

#[derive(Error, Debug, PartialEq)]
pub enum PauseError {
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
