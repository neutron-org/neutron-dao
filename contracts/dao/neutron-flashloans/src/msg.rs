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
    /// The main entry point of the flashloan contract. The caller of the `RequestLoan` message is
    /// expected to be a smart contract that implements the `BorrowerInterface`.
    ///
    /// This handler processes valiation of the request, records the loan details in storage, sends
    /// requested funds to the caller and then invokes the caller's `ProcessLoan` handler. After the
    /// `ProcessLoan` execution, the flashloans contract checks whether the borrower has done all
    /// required payments (loan + fee). If check fails, the entire transaction will be reverted.
    ///
    /// Borrower cannot request another loan until `ProcessLoan` execution has been finished.
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

/// Defines the interface for the borrowing contract — a contract that is capable of taking loans
/// from the flashloan contract.
#[cw_serde]
pub enum BorrowerInterface {
    /// The handler the borrower should place the loan usage logic within. At the time of this
    /// handler execution, the borrower will have the requested amount of funds received. The
    /// borrower must return the `loan_amount` plus the `fee` to the `return_address` by the end
    /// of this handler.
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
