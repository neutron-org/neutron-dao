use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

pub type ContractResult<T> = Result<T, ContractError>;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("cannot unlock {requested:?} funds when only {has:?} locked")]
    NotEnoughFundsToUnlock { requested: Uint128, has: Uint128 },

    #[error("denom is too short")]
    DenomTooShort,

    #[error("insufficient privileges: {0}")]
    InsufficientPrivileges(String),
}
