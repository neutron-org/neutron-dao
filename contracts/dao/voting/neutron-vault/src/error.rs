use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("config description cannot be empty.")]
    DescriptionIsEmpty {},

    #[error("config denom cannot be empty.")]
    DenomIsEmpty {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Nothing to claim")]
    NothingToClaim {},

    #[error("Only owner can change owner")]
    OnlyOwnerCanChangeOwner {},

    #[error("Can only unbond less than or equal to the amount you have bonded")]
    InvalidUnbondAmount {},
}
