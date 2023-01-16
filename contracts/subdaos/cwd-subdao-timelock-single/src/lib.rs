extern crate core;

pub mod contract;
mod error;
mod state;

#[cfg(test)]
mod testing;

pub use crate::error::ContractError;
