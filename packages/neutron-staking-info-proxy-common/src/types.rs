use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

/// Configuration.
#[cw_serde]
pub struct Config {
    /// The owner can update the contract's configuration and providers.
    pub owner: Addr,
    /// The contract address to which requests are proxied.
    pub staking_rewards: Option<Addr>,
    /// The denom used for staking rewards.
    pub staking_denom: String,
}

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        Ok(())
    }
}
