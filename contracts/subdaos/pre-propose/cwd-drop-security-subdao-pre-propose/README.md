# Single choice proposal deposit contract

This is a pre-propose module that manages proposal deposits for the
`cwd-proposal-single` proposal module.

It accepts NTRN tokens. If a proposal deposit is enabled (by default it is enabled)
the following refund strategies are avaliable:

1. Never refund deposits. All deposits are sent to the DAO on proposal
   completion.
2. Always refund deposits. Deposits are returned to the proposer on
   proposal completion.
3. Only refund passed proposals. Deposits are only returned to the
   proposer if the proposal passes. Otherwise, they are sent to the
   DAO.

This module may also be configured to only accept proposals from
members (addresses with voting power) of the DAO.
