use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Only owner can change owner")]
    OnlyOwnerCanChangeOwner {},

    #[error("Only owner can change lockdrop contract")]
    OnlyOwnerCanChangeLockdropContract {},

    #[error("config name cannot be empty.")]
    NameIsEmpty {},

    #[error("config description cannot be empty.")]
    DescriptionIsEmpty {},
}
