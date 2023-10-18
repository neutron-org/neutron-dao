use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Voting vault already exists")]
    VotingVaultAlreadyExists {},

    #[error("Voting vault is already in the active state")]
    VotingVaultAlreadyActive {},

    #[error("Voting vault is already in the inactive state")]
    VotingVaultAlreadyInactive {},
}
