use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, StdError, StdResult, Uint128};
use neutron_sdk::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::threshold::{validate_quorum, PercentageThreshold, ThresholdError};

/// Maximum number of choices for multiple choice votes
pub const MAX_NUM_CHOICES: u32 = 10;
const NONE_OPTION_DESCRIPTION: &str = "None of the above";

/// Determines how many choices may be selected.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VotingStrategy {
    SingleChoice { quorum: PercentageThreshold },
}

impl VotingStrategy {
    pub fn validate(&self) -> Result<(), ThresholdError> {
        match self {
            VotingStrategy::SingleChoice { quorum } => validate_quorum(quorum),
        }
    }

    pub fn get_quorum(&self) -> PercentageThreshold {
        match self {
            VotingStrategy::SingleChoice { quorum } => *quorum,
        }
    }
}

/// A multiple choice vote, picking the desired option
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema, Debug)]
pub struct MultipleChoiceVote {
    // A vote indicates which option the user has selected.
    pub option_id: u32,
}

impl std::fmt::Display for MultipleChoiceVote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.option_id)
    }
}

// Holds the vote weights for each option
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct MultipleChoiceVotes {
    // Vote counts is a vector of integers indicating the vote weight for each option
    // (the index corresponds to the option).
    pub vote_weights: Vec<Uint128>,
}

impl MultipleChoiceVotes {
    /// Sum of all vote weights
    pub fn total(&self) -> Uint128 {
        self.vote_weights.iter().sum()
    }

    // Add a vote to the tally
    pub fn add_vote(&mut self, vote: MultipleChoiceVote, weight: Uint128) -> StdResult<()> {
        self.vote_weights[vote.option_id as usize] = self.vote_weights[vote.option_id as usize]
            .checked_add(weight)
            .map_err(StdError::overflow)?;
        Ok(())
    }

    // Remove a vote from the tally
    pub fn remove_vote(&mut self, vote: MultipleChoiceVote, weight: Uint128) -> StdResult<()> {
        self.vote_weights[vote.option_id as usize] = self.vote_weights[vote.option_id as usize]
            .checked_sub(weight)
            .map_err(StdError::overflow)?;
        Ok(())
    }

    // Default tally of zero for all multiple choice options
    pub fn zero(num_choices: usize) -> Self {
        Self {
            vote_weights: vec![Uint128::zero(); num_choices],
        }
    }
}

/// Represents the type of Multiple choice option. "None of the above" has a special
/// type for example.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum MultipleChoiceOptionType {
    /// Choice that represents selecting none of the options; still counts toward quorum
    /// and allows proposals with all bad options to be voted against.
    None,
    Standard,
}

/// Represents unchecked multipl choice options
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MultipleChoiceOptions {
    pub options: Vec<MultipleChoiceOption>,
}

/// Unchecked multiple choice option
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MultipleChoiceOption {
    pub title: String,
    pub description: String,
    pub msgs: Option<Vec<CosmosMsg<NeutronMsg>>>,
}

/// Multiple choice options that have been verified for correctness, and have all fields
/// necessary for voting.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CheckedMultipleChoiceOptions {
    pub options: Vec<CheckedMultipleChoiceOption>,
}

/// A verified option that has all fields needed for voting.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CheckedMultipleChoiceOption {
    // This is the index of the option in both the vote_weights and proposal.choices vectors.
    // Workaround due to not being able to use HashMaps in Cosmwasm.
    pub index: u32,
    pub option_type: MultipleChoiceOptionType,
    pub title: String,
    pub description: String,
    pub msgs: Option<Vec<CosmosMsg<NeutronMsg>>>,
    pub vote_count: Uint128,
}

/// Deprecated.
/// This is the old choice option version without "title" field.
#[cw_serde]
pub struct OldCheckedMultipleChoiceOption {
    pub index: u32,
    pub option_type: MultipleChoiceOptionType,
    pub description: String,
    pub msgs: Option<Vec<CosmosMsg<NeutronMsg>>>,
    pub vote_count: Uint128,
}

impl MultipleChoiceOptions {
    pub fn into_checked(self) -> StdResult<CheckedMultipleChoiceOptions> {
        if self.options.len() < 2 || self.options.len() > MAX_NUM_CHOICES as usize {
            return Err(StdError::generic_err("Wrong number of choices".to_string()));
        }

        let mut checked_options: Vec<CheckedMultipleChoiceOption> =
            Vec::with_capacity(self.options.len() + 1);

        // Iterate through choices and save the index and option type for each
        self.options
            .into_iter()
            .enumerate()
            .for_each(|(idx, choice)| {
                let checked_option = CheckedMultipleChoiceOption {
                    index: idx as u32,
                    option_type: MultipleChoiceOptionType::Standard,
                    title: choice.title,
                    description: choice.description,
                    msgs: choice.msgs,
                    vote_count: Uint128::zero(),
                };
                checked_options.push(checked_option);
            });

        // Add a "None of the above" option, required for every multiple choice proposal.
        let none_option = CheckedMultipleChoiceOption {
            index: (checked_options.capacity() - 1) as u32,
            option_type: MultipleChoiceOptionType::None,
            title: NONE_OPTION_DESCRIPTION.to_string(),
            description: NONE_OPTION_DESCRIPTION.to_string(),
            msgs: None,
            vote_count: Uint128::zero(),
        };

        checked_options.push(none_option);

        let options = CheckedMultipleChoiceOptions {
            options: checked_options,
        };
        Ok(options)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_display_multiple_choice_vote() {
        let vote = MultipleChoiceVote { option_id: 0 };
        assert_eq!("0", vote.to_string())
    }

    #[test]
    fn test_multiple_choice_votes() {
        let mut votes = MultipleChoiceVotes {
            vote_weights: vec![Uint128::new(10), Uint128::new(100)],
        };
        let total = votes.total();
        assert_eq!(total, Uint128::new(110));

        votes
            .add_vote(MultipleChoiceVote { option_id: 0 }, Uint128::new(10))
            .unwrap();
        let total = votes.total();
        assert_eq!(total, Uint128::new(120));

        votes
            .remove_vote(MultipleChoiceVote { option_id: 0 }, Uint128::new(20))
            .unwrap();
        votes
            .remove_vote(MultipleChoiceVote { option_id: 1 }, Uint128::new(100))
            .unwrap();

        assert_eq!(votes, MultipleChoiceVotes::zero(2))
    }

    #[test]
    fn test_into_checked() {
        let options = vec![
            super::MultipleChoiceOption {
                title: "title".to_string(),
                description: "multiple choice option 1".to_string(),
                msgs: None,
            },
            super::MultipleChoiceOption {
                title: "title".to_string(),
                description: "multiple choice option 2".to_string(),
                msgs: None,
            },
        ];

        let mc_options = super::MultipleChoiceOptions { options };

        let checked_mc_options = mc_options.into_checked().unwrap();
        assert_eq!(checked_mc_options.options.len(), 3);
        assert_eq!(
            checked_mc_options.options[0].option_type,
            super::MultipleChoiceOptionType::Standard
        );
        assert_eq!(
            checked_mc_options.options[0].description,
            "multiple choice option 1",
        );
        assert_eq!(
            checked_mc_options.options[1].option_type,
            super::MultipleChoiceOptionType::Standard
        );
        assert_eq!(
            checked_mc_options.options[1].description,
            "multiple choice option 2",
        );
        assert_eq!(
            checked_mc_options.options[2].option_type,
            super::MultipleChoiceOptionType::None
        );
        assert_eq!(
            checked_mc_options.options[2].description,
            super::NONE_OPTION_DESCRIPTION,
        );
    }

    #[should_panic(expected = "Wrong number of choices")]
    #[test]
    fn test_into_checked_wrong_num_choices() {
        let options = vec![super::MultipleChoiceOption {
            title: "title".to_string(),
            description: "multiple choice option 1".to_string(),
            msgs: None,
        }];

        let mc_options = super::MultipleChoiceOptions { options };
        mc_options.into_checked().unwrap();
    }
}
