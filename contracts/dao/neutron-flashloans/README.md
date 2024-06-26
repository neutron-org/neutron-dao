# Neutron Flashloans

## Overview

The `neutron-flashloans` contract facilitates providing flash loans to smart contracts operating on the Neutron network.

A flash loan is a type of uncollateralized loan in the cryptocurrency and decentralized finance (DeFi) space. It allows
borrowers to borrow funds without providing any collateral, on the condition that the loan is repaid within the same
transaction. If the borrower fails to repay the loan by the end of the transaction, the entire transaction is
reversed, effectively canceling the loan. Flash loans are typically used for arbitrage, collateral swapping, and
refinancing, taking advantage of price discrepancies or temporary liquidity needs without requiring long-term capital.

## Usage

To get a flash loan, a `RequestLoan` message needs to be sent to the `neutron-flashloans` contract:

```rust
struct RequestLoan {
    /// The amount that the borrower contract requests; there should be no
    /// duplicate denoms and no zero amounts. 
    amount: Vec<Coin>,
}
```

The sender needs to be a smart-contract that implements a handler for the `ProcessLoan` message:

```rust
#[cw_serde]
pub enum BorrowerInterface {
    ProcessLoan {
        /// Specifies the address to which the borrower must return the loan amount AND pay the fees.
        return_address: Addr,
        /// Specifies the loan amount which the borrower must return to the return_address.
        loan_amount: Vec<Coin>,
        /// Specifies the fee which the borrower must pay to the return_address.
        fee: Vec<Coin>,
    }
}
```

Upon receiving the `RequestLoan` message, the `neutron-flashloans` contract will transfer the requested amount to the
borrower and send a `ProcessLoan` message. The borrower can execute any logic within its `ProcessLoan` handler but must
return the `loan_amount` plus the `fee` to the `return_address`. Failure to do so will result in the entire transaction
being reverted.

## Implementation

The `neutron-flashloans` contract does not hold any funds. Instead, it uses `authz` permission from the `source` address
to execute `/cosmos.bank.v1beta1.MsgSend` on its behalf. For Neutron, the `source` address must be set to the Treasury (
DAO core) contract address.

* The `RequestLoan` handler ensures there is no active loan, validates the loan amount (no duplicate or zero coins),
  calculates the expected balance (current balance + fee) of the source after repayment, and records the loan details in
  storage. If the `source` does not have the requested amount of funds, an error will be returned. Finally, it instructs
  the source to send the requested amount to the borrower via `authz`, encapsulated in a `stargate message`. This
  message is submitted as a submessage with a `reply_on_success` strategy, meaning if it fails, the transaction is
  reverted.
* Upon successful execution of the `/cosmos.bank.v1beta1.MsgSend` message, the `neutron-flashloans` contract sends
  a `ProcessLoan` submessage with a `reply_on_success` strategy to the borrower contract.
* After receiving a successful reply to the `ProcessLoan` message, the `neutron-flashloans` contract verifies that the
  borrower has returned the funds and paid the fee, then it deletes the loan information.

## Security advice

When writing a borrower contract, ensure that the `ProcessLoan` handler has proper permissions. It should only be
callable when your contract has previously requested a loan and only by the `neutron-flashloans` contract.
