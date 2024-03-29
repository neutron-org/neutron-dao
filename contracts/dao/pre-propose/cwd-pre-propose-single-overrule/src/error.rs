use cosmwasm_std::StdError;
use cwd_pre_propose_base::error::PreProposeError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PreProposeOverruleError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    PreProposeBase(PreProposeError),

    #[error("Base pre propose messages aren't supported")]
    MessageUnsupported {},

    #[error("Subdao is misconfigured")]
    SubdaoMisconfigured {},

    #[error("Subdao isn't in the list")]
    ForbiddenSubdao {},

    #[error("Subdao proposal isn't timelocked")]
    ProposalWrongState {},

    #[error("Overrule proposal for this subdao proposal already created with id ({id})")]
    AlreadyExists { id: u64 },
}
