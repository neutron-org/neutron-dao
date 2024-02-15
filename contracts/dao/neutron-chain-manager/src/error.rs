use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("MissingInitialStrategy")]
    InvalidInitialStrategy {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidDemotion")]
    InvalidDemotion {},

    // A variant for serde_json_wasm deserialization errors.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl From<serde_json_wasm::de::Error> for ContractError {
    fn from(err: serde_json_wasm::de::Error) -> ContractError {
        ContractError::DeserializationError(err.to_string())
    }
}