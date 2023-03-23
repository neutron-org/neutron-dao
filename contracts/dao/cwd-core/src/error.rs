use cosmwasm_std::{Addr, StdError};
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    ParseReplyError(#[from] ParseReplyError),

    #[error("config name cannot be empty.")]
    NameIsEmpty {},

    #[error("config description cannot be empty.")]
    DescriptionIsEmpty {},

    #[error("config DAO URI cannot be empty.")]
    DaoUriIsEmpty {},

    #[error("Unauthorized.")]
    Unauthorized {},

    #[error("The contract is paused.")]
    Paused {},

    #[error("Execution would result in no proposal modules being active.")]
    NoActiveProposalModules {},

    #[error("An unknown reply ID was received.")]
    UnknownReplyID {},

    #[error("Multiple voting modules during instantiation.")]
    MultipleVotingModules {},

    #[error("Key is missing from storage")]
    KeyMissing {},

    #[error("Proposal module with address ({address}) does not exist.")]
    ProposalModuleDoesNotExist { address: Addr },

    #[error("Proposal module with address ({address}) is already disabled.")]
    ModuleAlreadyDisabled { address: Addr },

    #[error("Proposal module with address is disabled and cannot execute messages.")]
    ModuleDisabledCannotExecute { address: Addr },
}
