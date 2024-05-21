use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RequestLoan {
        /// Address to get the flashloans from.
        flashloans_contract: Addr,

        /// Determines how the loan should be processed.
        ///
        /// MODE_RETURN_LOAN = 0 (return the correct amount)
        /// MODE_WITHHOLD_LOAN = 1 (not return anything)
        /// MODE_RETURN_LOAN_MORE_THAN_NECESSARY = 2 (return more than expected)
        /// MODE_REQUEST_ANOTHER_LOAN_RECURSIVELY = 3 (request another loan while processing the existing loan)
        ///
        /// Any other value will result in returning an error.
        execution_mode: u64,

        /// The amount to request.
        amount: Vec<Coin>,
    },
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
pub enum QueryMsg {}

#[cw_serde]
pub enum FlashloansExecuteMsg {
    /// This message duplicates the neutron-flashloan RequestLoan message
    /// to avoid moving it to packages.
    RequestLoan { amount: Vec<Coin> },
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
