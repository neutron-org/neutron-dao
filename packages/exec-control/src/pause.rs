use cosmwasm_std::Addr;
use thiserror::Error;

/// Approximately one week given block time = 2sec.
pub const MAX_PAUSE_DURATION: u64 = 302400;

// checks whether the sender is capable to pause a contract
pub fn can_pause(
    sender: &Addr,
    main_dao_address: &Addr,
    security_dao_address: &Addr,
) -> Result<(), PauseError> {
    let authorized = sender == main_dao_address || sender == security_dao_address;

    if !authorized {
        return Err(PauseError::Unauthorized {});
    }

    Ok(())
}

// checks whether the sender is capable to unpause a contract
pub fn can_unpause(sender: &Addr, main_dao_address: &Addr) -> Result<(), PauseError> {
    if sender != main_dao_address {
        return Err(PauseError::Unauthorized {});
    }

    Ok(())
}

/// checks whether the duration is not greater than MAX_PAUSE_DURATION.
pub fn validate_duration(duration: u64) -> Result<(), PauseError> {
    if duration.gt(&MAX_PAUSE_DURATION) {
        return Err(PauseError::InvalidDuration(format!(
            "Pause duration is too big: it's only possible to pause the contract for {} blocks",
            MAX_PAUSE_DURATION
        )));
    }
    Ok(())
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PauseError {
    #[error("Unauthorized.")]
    Unauthorized {},

    #[error("invalid duration")]
    InvalidDuration(String),

    #[error("Contract execution is paused.")]
    Paused {},
}
