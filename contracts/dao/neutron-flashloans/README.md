# Neutron Flashloans

## Overview

A flash loan is a type of uncollateralized loan in the cryptocurrency and decentralized finance (DeFi) space. It allows
borrowers to borrow funds without providing any collateral, on the condition that the loan is repaid within the same
transaction. If the borrower fails to repay the loan by the end of the transaction, the entire transaction is
reversed, effectively canceling the loan. Flash loans are typically used for arbitrage, collateral swapping, and
refinancing, taking advantage of price discrepancies or temporary liquidity needs without requiring long-term capital.

The `neutron-flashloans` contract facilitates providing flash loans to smart contracts operating on the Neutron network.

## Usage

See the `neutron-flashloans` contract's [interface](https://github.com/neutron-org/neutron-dao/blob/main/contracts/dao/neutron-flashloans/src/msg.rs) in order to get familiar with all requirements, limitations and usage guidelines.

See the [neutron-flashloans-user](https://github.com/neutron-org/neutron-dev-contracts/blob/main/contracts/neutron-flashloans-user) contract as an example of the borrower contract implementation.

### Security advice

When writing a borrower contract, ensure that the `ProcessLoan` handler has proper permissions. It should only be
callable when your contract has previously requested a loan and only by the `neutron-flashloans` contract.
