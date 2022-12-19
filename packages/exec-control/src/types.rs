use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Approximately one week given block time = 2sec.
pub const MAX_PAUSE_DURATION: u64 = 302400;

/// The list of actions considered by the execution control management kit.
pub enum PausedStateAction {
    Pause(u64),
    Unpause,
    Other,
}

/// Information about if the contract is currently paused.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum PauseInfoResponse {
    Paused { until_height: u64 },
    Unpaused {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    /// The address of the contract admin.
    pub admin: Addr,
    /// The address of the contract guardian. If defined, the guardian is capable of pausing and
    /// unpausing the contract.
    pub guardian: Option<Addr>,
}
