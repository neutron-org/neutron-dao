# Neutron Flashloans

## Overview

This contract provides flashloans, facilitating [authz](https://docs.cosmos.network/main/build/modules/authz) permissions to sending funds on behalf of the `source` — actual funds owner. In order to let the flashloans contract use the source's funds, the source must [grant](https://docs.cosmos.network/main/build/modules/authz#msggrant) `/cosmos.bank.v1beta1.MsgSend` permissions to the flashloans contract.

## Interaction

Here's a brief description of what happens when a borrower requests a loan:
1. The borrower sends a `RequestLoan` message to the flashloan contract;
2. The flashloan contract validates the request and sends the requested amount of funds to the borrower from the source;
3. The flashloan contract completes the send message with the `reply` callback by sending a `ProcessLoan` message to the borrower;
4. Inside `ProcessLoan`, the borrower processes its custom logic with the loan and refunds the loan amount plus fee to the source;
5. The flashloan contract completes the `ProcessLoan` message with the `reply` callback by confirming the loan amount plus fee refund to the source.

See the `neutron-flashloans` contract's [interface](https://github.com/neutron-org/neutron-dao/blob/main/contracts/dao/neutron-flashloans/src/msg.rs) in order to get familiar with all requirements, limitations and usage guidelines.

See the [neutron-flashloans-user](https://github.com/neutron-org/neutron-dev-contracts/blob/main/contracts/neutron-flashloans-user) contract as an example of the borrower contract implementation.

### Security advice

When writing a borrower contract, ensure that the `ProcessLoan` handler has proper permissions. It should only be
callable when your contract has previously requested a loan and only by the `neutron-flashloans` contract.
