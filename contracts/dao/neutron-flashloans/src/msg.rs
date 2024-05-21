use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal};

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Defines the owner of the contract. The owner can add and remove
    /// sources from the contract.
    pub owner: Addr,
    /// Defines the address that is going to sponsor the loans. This address
    /// needs to grant a (Generic)Authorization to this contract to execute
    /// /cosmos.bank.v1beta1.MsgSend on its behalf.
    pub source: Addr,
    /// Defines the fee rate for the loans, e.g., fee_rate = 0.01
    /// means that if you borrow 100untrn, you'll need to return 101untrn.
    pub fee_rate: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    RequestLoan {
        /// The amount that the borrower contract requests; there should be no
        /// duplicate denoms and no zero amounts.
        amount: Vec<Coin>,
    },
    UpdateConfig {
        owner: Option<Addr>,
        fee_rate: Option<Decimal>,
        source: Option<Addr>,
    },
}

/// Defines the interface for the borrowing contract. The borrowing
/// contract is required to implement a handler for the ProcessLoan message,
/// and to return loan_amount + fee to the return_address after executing its
/// custom logic, otherwise an error will be raised, and the whole transaction
/// will be rolled back.
#[cw_serde]
pub enum BorrowerInterface {
    ProcessLoan {
        /// Specifies the address to which the borrower must return the loan amount AND pay the fees.
        return_address: Addr,
        /// Specifies the loan amount which the borrower must return to the return_address.
        loan_amount: Vec<Coin>,
        /// Specifies the fee which the borrower must pay to the return_address.
        fee: Vec<Coin>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current config value.
    #[returns(Config)]
    Config {},
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
