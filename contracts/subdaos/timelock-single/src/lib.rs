extern crate core;

pub mod contract;
mod error;
pub mod msg;
pub mod proposal;
pub mod state;

#[cfg(test)]
mod tests;

pub use crate::error::ContractError;
