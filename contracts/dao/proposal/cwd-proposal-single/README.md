# cwd-proposal-single

A proposal module for a Neutron DAO which supports simple "yes", "no",
"abstain" voting. Proposals may have associated messages which will be
executed by the core module upon the proposal being passed and
executed.

## Proposal deposits

Proposal deposits for this module are handled by the
[`cwd-pre-propose-single`](../../pre-propose/cwd-pre-propose-single)
contract.

## Hooks

This module supports hooks for voting and proposal status changes. One
may register a contract to receive these hooks with the `AddVoteHook`
and `AddProposalHook` methods. Upon registration the contract will
receive messages whenever a vote is cast and a proposal's status
changes (for example, when the proposal passes).

The format for these hook messages can be located in the
`proposal-hooks` and `vote-hooks` packages located in
`packages/proposal-hooks` and `packages/vote-hooks` respectively.

To stop an invalid hook receiver from locking the proposal module
receivers will be removed from the hook list if they error when
handling a hook.
