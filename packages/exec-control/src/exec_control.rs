use crate::error::ExecControlError;
use crate::state::{CONFIG, PAUSED_UNTIL};
use crate::types::{Config, PauseInfoResponse, PausedStateAction, MAX_PAUSE_DURATION};
use cosmwasm_std::{Addr, Env, MessageInfo, Response, StdResult, Storage};
use neutron_bindings::bindings::msg::NeutronMsg;

/// Initializes storage items needed for execution control.
pub fn init(store: &mut dyn Storage, cfg: &Config) -> Result<(), ExecControlError> {
    CONFIG.save(store, cfg)?;
    PAUSED_UNTIL.save(store, &None)?;
    Ok(())
}

/// Checks whether the contract is paused and acts as follows:
///
/// * if paused: allows only `Pause` and `Unpause` actions and performs them. If the `Other` action
/// is received in the paused state, it returns a `ExecControlError::Paused`.
/// * if not paused: returns a `Ok(None)` to let the caller contract finish the execution.
pub fn handle_possible_paused_state(
    store: &mut dyn Storage,
    env: Env,
    info: MessageInfo,
    action: PausedStateAction,
) -> Result<Option<Response<NeutronMsg>>, ExecControlError> {
    match get_pause_info(store, env.clone())? {
        PauseInfoResponse::Paused { .. } => match action {
            PausedStateAction::Pause(duration) => {
                execute_pause(store, env, info.sender, duration).map(Some)
            }
            PausedStateAction::Unpause {} => execute_unpause(store, info.sender).map(Some),
            PausedStateAction::Other {} => Err(ExecControlError::Paused {}),
        },
        PauseInfoResponse::Unpaused {} => Ok(None),
    }
}

/// Returns information about the contract's current execution control state.
pub fn get_pause_info(store: &dyn Storage, env: Env) -> StdResult<PauseInfoResponse> {
    Ok(match PAUSED_UNTIL.load(store)? {
        Some(paused_until_height) => {
            if env.block.height.ge(&paused_until_height) {
                PauseInfoResponse::Unpaused {}
            } else {
                PauseInfoResponse::Paused {
                    until_height: paused_until_height,
                }
            }
        }
        None => PauseInfoResponse::Unpaused {},
    })
}

/// Sets the contract's state to the paused state.
pub fn execute_pause(
    store: &mut dyn Storage,
    env: Env,
    sender: Addr,
    duration: u64,
) -> Result<Response<NeutronMsg>, ExecControlError> {
    let config: Config = CONFIG.load(store)?;
    validate_sender(sender.clone(), config.admin, config.guardian)?;
    validate_duration(duration)?;

    let paused_until_height: u64 = env.block.height + duration;
    PAUSED_UNTIL.save(store, &Some(paused_until_height))?;

    Ok(Response::new()
        .add_attribute("action", "execute_pause")
        .add_attribute("sender", sender)
        .add_attribute("paused_until_height", paused_until_height.to_string()))
}

/// Unsets the contract's paused state.
pub fn execute_unpause(
    store: &mut dyn Storage,
    sender: Addr,
) -> Result<Response<NeutronMsg>, ExecControlError> {
    let config: Config = CONFIG.load(store)?;
    validate_sender(sender.clone(), config.admin, config.guardian)?;

    PAUSED_UNTIL.save(store, &None)?;

    Ok(Response::new()
        .add_attribute("action", "execute_unpause")
        .add_attribute("sender", sender))
}

/// checks whether the sender is either the admin or the guardian if any.
fn validate_sender(
    sender: Addr,
    admin: Addr,
    guardian: Option<Addr>,
) -> Result<(), ExecControlError> {
    let authorized = match guardian {
        Some(g) => sender == admin || sender == g,
        None => sender == admin,
    };
    if !authorized {
        return Err(ExecControlError::Unauthorized {});
    }
    Ok(())
}

/// checks whether the duration is not greater than MAX_PAUSE_DURATION.
fn validate_duration(duration: u64) -> Result<(), ExecControlError> {
    if duration.gt(&MAX_PAUSE_DURATION) {
        return Err(ExecControlError::PauseDurationTooBig {});
    }
    Ok(())
}
