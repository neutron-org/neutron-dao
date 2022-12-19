extern crate core;

pub mod contract;
mod error;
pub mod msg;
pub mod state;
pub mod proposal;

#[cfg(test)]
mod tests;

pub use crate::error::ContractError;
