use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("neutron-flashloans-config");

/// ACTIVE_LOAN keeps the information about the current loan, and acts as a reentrancy guard.
pub const ACTIVE_LOAN: Item<Option<ActiveLoan>> = Item::new("neutron-flashloans-active-loan");

#[cw_serde]
pub struct Config {
    /// Defines the address that can modify the configuration of the contract.
    pub owner: Addr,
    /// Defines the fee rate for the loans, e.g., fee_rate = 0.01
    /// means that if you borrow 100untrn, you'll need to return 101untrn.
    pub fee_rate: Decimal,
    /// Defines the address that is going to sponsor the loans. This address
    /// needs to grant a (Generic)Authorization to this contract to execute
    /// /cosmos.bank.v1beta1.MsgSend on its behalf.
    pub source: Addr,
}

/// Defines the information that we store about the active loan.
#[cw_serde]
pub struct ActiveLoan {
    /// The borrowing contract.
    pub borrower: Addr,
    /// The amount that was requested by the borrowing contract.
    pub amount: Vec<Coin>,
    /// The fee that needs to be returned by the borrowing contract.
    pub fee: Vec<Coin>,
    /// The expected balances of the source contract after the borrowing contract
    /// returns (loan amount + fees).
    pub expected_balances: Vec<Coin>,
}
