use std::u64;

use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use cwd_hooks::HookError;
use cwd_voting::reply::error::TagError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    ParseReplyError(#[from] ParseReplyError),

    #[error(transparent)]
    HookError(#[from] HookError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error(transparent)]
    ThresholdError(#[from] cwd_voting::threshold::ThresholdError),

    #[error(transparent)]
    VotingError(#[from] cwd_voting::error::VotingError),

    #[error("no such proposal ({id})")]
    NoSuchProposal { id: u64 },

    #[error("proposal is ({size}) bytes, must be <= ({max}) bytes")]
    ProposalTooLarge { size: u64, max: u64 },

    #[error("proposal is not open ({id})")]
    NotOpen { id: u64 },

    #[error("not registered to vote (no voting power) at time of proposal creation")]
    NotRegistered {},

    #[error("already voted. this proposal does not support revoting")]
    AlreadyVoted {},

    #[error("already cast a vote with that option. change your vote to revote")]
    AlreadyCast {},

    #[error("proposal is not in 'passed' state")]
    NotPassed {},

    #[error("only rejected proposals may be closed")]
    WrongCloseStatus {},

    #[error(
        "pre-propose modules must specify a proposer. lacking one, no proposer should be specified"
    )]
    InvalidProposer {},

    #[error(transparent)]
    Tag(#[from] TagError),

    #[error("received a reply failure with an invalid ID: ({id})")]
    InvalidReplyID { id: u64 },
}
