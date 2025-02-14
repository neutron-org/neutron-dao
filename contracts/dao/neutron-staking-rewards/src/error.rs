use cosmwasm_std::{CheckedMultiplyFractionError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Insufficient contract balance to pay rewards")]
    InsufficientBalance {},

    #[error("No stake changes allowed for DAO address")]
    DaoStakeChangeNotTracked {},

    #[error("Invalid stake denom returned from staking info proxy contract: {denom}")]
    InvalidStakeDenom { denom: String },

    #[error("Empty stake denom provided")]
    EmptyStakeDenom {},

    #[error("Zero blocks per year provided")]
    ZeroBlocksPerYear {},

    #[error("CheckedMultiplyRatioError error: {0}")]
    CheckedMultiplyFractionError(#[from] CheckedMultiplyFractionError),

    #[error("IncorrectGlobalIndexHeightToUpdate error: current height {current_block} height to update {last_global_update_block}")]
    IncorrectGlobalIndexHeightToUpdate {
        current_block: u64,
        last_global_update_block: u64,
    },
}
