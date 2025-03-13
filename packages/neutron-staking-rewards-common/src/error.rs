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

    #[error("Invalid annual reward rate bps: {bps} must be in [0;10000] range")]
    InvalidBPS {bps: u64},

    #[error("CheckedMultiplyRatioError error: {0}")]
    CheckedMultiplyFractionError(#[from] CheckedMultiplyFractionError),

    #[error("TriedGetGlobalIndexInThePast error: current height is {current_block}, height to update is {last_global_update_block}")]
    TriedGetGlobalIndexInThePast {
        current_block: u64,
        last_global_update_block: u64,
    },
}
