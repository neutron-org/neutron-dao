use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("config name cannot be empty.")]
    NameIsEmpty {},

    #[error("config description cannot be empty.")]
    DescriptionIsEmpty {},
}

pub type ContractResult<T> = Result<T, ContractError>;
