use cosmwasm_std::StdError;
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

    #[error("Invalid stake denom returned from staking info proxy contract")]
    InvalidStakeDenom {},

    #[error("Empty stake denom provided")]
    EmptyStakeDenom {},

    #[error("Zero blocks per year provided")]
    ZeroBlocksPerYear {},
}
