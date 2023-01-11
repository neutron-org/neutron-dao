use thiserror::Error;
use cwd_pre_propose_base::error::PreProposeError;

#[derive(Error, Debug, PartialEq)]
pub enum PreProposeOverruleError {
    #[error(transparent)]
    PreProposeBase(PreProposeError),

    #[error("Base pre propose messages aren't supported.")]
    MessageUnsupported {},
}