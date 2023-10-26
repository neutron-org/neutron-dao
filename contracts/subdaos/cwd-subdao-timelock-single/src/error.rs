use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Only owner can change owner")]
    OnlyOwnerCanChangeOwner {},

    #[error("Proposal is timelocked")]
    TimeLocked {},

    #[error("Wrong proposal status ({status})")]
    WrongStatus { status: String },

    #[error("No such proposal ({id})")]
    NoSuchProposal { id: u64 },

    #[error("Can not create overrule proposal for main DAO")]
    CantCreateOverrule {},

    #[error("Can only execute proposals with exactly one message that of ExecuteTimelockedMsgs type. Got {len} messages.")]
    CanOnlyExecuteOneMsg { len: usize },

    #[error("Can only execute msg of ExecuteTimelockedMsgs type")]
    CanOnlyExecuteExecuteTimelockedMsgs {},
}
