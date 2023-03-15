use cosmwasm_std::StdError;
use cwd_pre_propose_base::error::PreProposeError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PreProposeOverruleError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    PreProposeBase(PreProposeError),

    #[error("Base pre propose messages aren't supported.")]
    MessageUnsupported {},

    #[error("Subdao is wrongly configured.")]
    SubdaoMisconfured {},

    #[error("Subdao isn't in the list.")]
    ForbiddenSubdao {},

    #[error("")]
    ProposalWrongState {},

    #[error("")]
    AlreadyExists {},
}
