use cosmwasm_std::{
    coins, to_binary, Addr, Api, Coin, CosmosMsg, Decimal, Empty, Storage, Timestamp, Uint128,
    WasmMsg,
};
use cw20::Cw20Coin;
use cw_denom::{CheckedDenom, UncheckedDenom};
use cw_multi_test::{
    custom_app, next_block, BankSudo, BasicApp, Contract, ContractWrapper, Executor, Router,
    SudoMsg,
};
use cw_utils::Duration;
use cwd_core::state::ProposalModule;
use cwd_hooks::HooksResponse;
use cwd_interface::{Admin, ModuleInstantiateInfo};
use cwd_voting::{
    deposit::{CheckedDepositInfo, DepositRefundPolicy, DepositToken, UncheckedDepositInfo},
    multiple_choice::{
        CheckedMultipleChoiceOption, MultipleChoiceOption, MultipleChoiceOptionType,
        MultipleChoiceOptions, MultipleChoiceVote, MultipleChoiceVotes, VotingStrategy,
        MAX_NUM_CHOICES,
    },
    pre_propose::PreProposeInfo,
    status::Status,
    threshold::PercentageThreshold,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use std::panic;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    proposal::MultipleChoiceProposal,
    query::{ProposalListResponse, ProposalResponse},
    state::Config,
    testing::{
        execute::make_proposal,
        instantiate::instantiate_with_native_bonded_balances_governance,
        queries::{
            query_balance_native, query_deposit_config_and_pre_propose_module,
            query_list_proposals, query_list_proposals_reverse, query_multiple_proposal_module,
            query_proposal, query_proposal_config, query_proposal_hooks, query_vote_hooks,
        },
    },
    ContractError,
};
use cwd_pre_propose_multiple as cppm;

use crate::testing::execute::mint_natives;
use cwd_testing::ShouldExecute;

pub const CREATOR_ADDR: &str = "creator";

pub struct TestMultipleChoiceVote {
    /// The address casting the vote.
    pub voter: String,
    /// Position on the vote.
    pub position: MultipleChoiceVote,
    /// Voting power of the address.
    pub weight: Uint128,
    /// If this vote is expected to execute.
    pub should_execute: ShouldExecute,
}

pub(crate) fn no_init<BankT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT>(
    _: &mut Router<BankT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT>,
    _: &dyn Api,
    _: &mut dyn Storage,
) {
}

pub fn proposal_multiple_contract() -> Box<dyn Contract<NeutronMsg>> {
    let contract: ContractWrapper<_, _, _, _, _, _, NeutronMsg, _, _, _, _, _, _> =
        ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply_empty(crate::contract::reply);
    Box::new(contract)
}

pub fn pre_propose_multiple_contract() -> Box<dyn Contract<NeutronMsg>> {
    let contract: ContractWrapper<_, _, _, _, _, _, NeutronMsg, _, _, _, _, _, _> =
        ContractWrapper::new_with_empty(
            cppm::contract::execute,
            cppm::contract::instantiate,
            cppm::contract::query,
        );
    Box::new(contract)
}

pub fn get_pre_propose_info(
    app: &mut BasicApp<NeutronMsg>,
    deposit_info: Option<UncheckedDepositInfo>,
    open_proposal_submission: bool,
) -> PreProposeInfo {
    let pre_propose_contract = app.store_code(pre_propose_multiple_contract());
    PreProposeInfo::ModuleMayPropose {
        info: ModuleInstantiateInfo {
            code_id: pre_propose_contract,
            msg: to_binary(&cppm::InstantiateMsg {
                deposit_info,
                open_proposal_submission,
            })
            .unwrap(),
            admin: Some(Admin::CoreModule {}),
            label: "pre_propose_contract".to_string(),
        },
    }
}

#[test]
fn test_propose() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());

    let max_voting_period = cw_utils::Duration::Height(6);
    let quorum = PercentageThreshold::Majority {};

    let voting_strategy = VotingStrategy::SingleChoice { quorum };

    let instantiate = InstantiateMsg {
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        voting_strategy: voting_strategy.clone(),
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let govmod = query_multiple_proposal_module(&app, &core_addr);

    // Check that the config has been configured correctly.
    let config: Config = query_proposal_config(&app, &govmod);
    let expected = Config {
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        dao: core_addr,
        voting_strategy: voting_strategy.clone(),
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
    };
    assert_eq!(config, expected);

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
    ];

    let mc_options = MultipleChoiceOptions { options };

    // Create a new proposal.
    make_proposal(&mut app, &govmod, CREATOR_ADDR, mc_options.clone());

    let created: ProposalResponse = query_proposal(&app, &govmod, 1);

    let current_block = app.block_info();
    let checked_options = mc_options.into_checked().unwrap();
    let expected = MultipleChoiceProposal {
        title: "title".to_string(),
        description: "description".to_string(),
        proposer: Addr::unchecked(CREATOR_ADDR),
        start_height: current_block.height,
        expiration: max_voting_period.after(&current_block),
        choices: checked_options.options,
        status: Status::Open,
        voting_strategy,
        total_power: Uint128::new(100_000_000),
        votes: MultipleChoiceVotes {
            vote_weights: vec![Uint128::zero(); 3],
        },
        allow_revoting: false,
        min_voting_period: None,
    };

    assert_eq!(created.proposal, expected);
    assert_eq!(created.id, 1u64);
}

#[test]
fn test_propose_wrong_num_choices() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());

    let max_voting_period = cw_utils::Duration::Height(6);
    let quorum = PercentageThreshold::Majority {};

    let voting_strategy = VotingStrategy::SingleChoice { quorum };

    let instantiate = InstantiateMsg {
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        voting_strategy: voting_strategy.clone(),
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let govmod = query_multiple_proposal_module(&app, &core_addr);

    // Check that the config has been configured correctly.
    let config: Config = query_proposal_config(&app, &govmod);
    let expected = Config {
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        dao: core_addr,
        voting_strategy,
    };
    assert_eq!(config, expected);

    let options = vec![];

    // Create a proposal with less than min choices.
    let mc_options = MultipleChoiceOptions { options };
    let err = app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    );
    assert!(err.is_err());

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        };
        std::convert::TryInto::try_into(MAX_NUM_CHOICES + 1).unwrap()
    ];

    // Create proposal with more than max choices.

    let mc_options = MultipleChoiceOptions { options };
    // Create a new proposal.
    let err = app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod,
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    );
    assert!(err.is_err());
}

#[test]
fn test_proposal_count_initialized_to_zero() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _proposal_id = app.store_code(proposal_multiple_contract());
    let msg = InstantiateMsg {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(10)),
        },
        max_voting_period: Duration::Height(10),
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        only_members_execute: true,
        allow_revoting: false,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };
    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, msg, None);

    let gov_state: cwd_core::query::DumpStateResponse = app
        .wrap()
        .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
        .unwrap();
    let proposal_modules = gov_state.proposal_modules;

    assert_eq!(proposal_modules.len(), 1);
    let govmod = proposal_modules.into_iter().next().unwrap().address;

    let proposal_count: u64 = app
        .wrap()
        .query_wasm_smart(govmod, &QueryMsg::ProposalCount {})
        .unwrap();

    assert_eq!(proposal_count, 0);
}

#[test]
fn test_no_early_pass_with_min_duration() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let msg = InstantiateMsg {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(10)),
        },
        max_voting_period: Duration::Height(10),
        min_voting_period: Some(Duration::Height(2)),
        only_members_execute: true,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        msg,
        Some(vec![
            Cw20Coin {
                address: "blue".to_string(),
                amount: Uint128::new(10),
            },
            Cw20Coin {
                address: "whale".to_string(),
                amount: Uint128::new(90),
            },
        ]),
    );

    let gov_state: cwd_core::query::DumpStateResponse = app
        .wrap()
        .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
        .unwrap();
    let proposal_modules = gov_state.proposal_modules;

    assert_eq!(proposal_modules.len(), 1);
    let govmod = proposal_modules.into_iter().next().unwrap().address;

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];

    let mc_options = MultipleChoiceOptions { options };

    app.execute_contract(
        Addr::unchecked("whale"),
        govmod.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "This is a simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // Whale votes which under normal curcumstances would cause the
    // proposal to pass. Because there is a min duration it does not.
    app.execute_contract(
        Addr::unchecked("whale"),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);

    assert_eq!(proposal.proposal.status, Status::Open);

    // Let the min voting period pass.
    app.update_block(|b| b.height += 2);

    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);

    assert_eq!(proposal.proposal.status, Status::Passed);
}

#[test]
fn test_propose_with_messages() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let msg = InstantiateMsg {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(10)),
        },
        max_voting_period: Duration::Height(10),
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        only_members_execute: true,
        allow_revoting: false,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        msg,
        Some(vec![
            Cw20Coin {
                address: "blue".to_string(),
                amount: Uint128::new(10),
            },
            Cw20Coin {
                address: "whale".to_string(),
                amount: Uint128::new(90),
            },
        ]),
    );

    let gov_state: cwd_core::query::DumpStateResponse = app
        .wrap()
        .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
        .unwrap();
    let proposal_modules = gov_state.proposal_modules;

    assert_eq!(proposal_modules.len(), 1);
    let govmod = proposal_modules.into_iter().next().unwrap().address;

    let config_msg = ExecuteMsg::UpdateConfig {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Majority {},
        },
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        max_voting_period: cw_utils::Duration::Height(20),
        only_members_execute: false,
        allow_revoting: false,
        dao: "dao".to_string(),
    };

    let wasm_msg = WasmMsg::Execute {
        contract_addr: govmod.to_string(),
        msg: to_binary(&config_msg).unwrap(),
        funds: vec![],
    };

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: Some(vec![CosmosMsg::Wasm(wasm_msg)]),
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];

    let mc_options = MultipleChoiceOptions { options };

    app.execute_contract(
        Addr::unchecked("whale"),
        govmod.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "This is a simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("whale"),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);

    assert_eq!(proposal.proposal.status, Status::Passed);

    // Execute the proposal and messages
    app.execute_contract(
        Addr::unchecked("blue"),
        govmod.clone(),
        &ExecuteMsg::Execute { proposal_id: 1 },
        &[],
    )
    .unwrap();

    // Check that config was updated by proposal message
    let config: Config = query_proposal_config(&app, &govmod);
    assert_eq!(config.max_voting_period, Duration::Height(20))
}

#[test]
#[should_panic(
    expected = "min_voting_period and max_voting_period must have the same units (height or time)"
)]
fn test_min_duration_units_missmatch() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let msg = InstantiateMsg {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(10)),
        },
        max_voting_period: Duration::Height(10),
        min_voting_period: Some(Duration::Time(2)),
        only_members_execute: true,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };
    instantiate_with_native_bonded_balances_governance(
        &mut app,
        msg,
        Some(vec![
            Cw20Coin {
                address: "blue".to_string(),
                amount: Uint128::new(10),
            },
            Cw20Coin {
                address: "wale".to_string(),
                amount: Uint128::new(90),
            },
        ]),
    );
}

#[test]
#[should_panic(expected = "Min voting period must be less than or equal to max voting period")]
fn test_min_duration_larger_than_proposal_duration() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let msg = InstantiateMsg {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(10)),
        },
        max_voting_period: Duration::Height(10),
        min_voting_period: Some(Duration::Height(11)),
        only_members_execute: true,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };
    instantiate_with_native_bonded_balances_governance(
        &mut app,
        msg,
        Some(vec![
            Cw20Coin {
                address: "blue".to_string(),
                amount: Uint128::new(10),
            },
            Cw20Coin {
                address: "wale".to_string(),
                amount: Uint128::new(90),
            },
        ]),
    );
}

#[test]
fn test_min_duration_same_as_proposal_duration() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let msg = InstantiateMsg {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(10)),
        },
        max_voting_period: Duration::Time(10),
        min_voting_period: Some(Duration::Time(10)),
        only_members_execute: true,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        msg,
        Some(vec![
            Cw20Coin {
                address: "blue".to_string(),
                amount: Uint128::new(10),
            },
            Cw20Coin {
                address: "whale".to_string(),
                amount: Uint128::new(90),
            },
        ]),
    );

    let gov_state: cwd_core::query::DumpStateResponse = app
        .wrap()
        .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
        .unwrap();
    let proposal_modules = gov_state.proposal_modules;

    assert_eq!(proposal_modules.len(), 1);
    let govmod = proposal_modules.into_iter().next().unwrap().address;

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];

    let mc_options = MultipleChoiceOptions { options };

    app.execute_contract(
        Addr::unchecked("whale"),
        govmod.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "This is a simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // Whale votes which under normal curcumstances would cause the
    // proposal to pass. Because there is a min duration it does not.
    app.execute_contract(
        Addr::unchecked("whale"),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);

    assert_eq!(proposal.proposal.status, Status::Open);

    // someone else can vote none of the above.
    app.execute_contract(
        Addr::unchecked("blue"),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 2 },
        },
        &[],
    )
    .unwrap();

    // Let the min voting period pass.
    app.update_block(|b| b.time = b.time.plus_seconds(10));

    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);

    assert_eq!(proposal.proposal.status, Status::Passed);
}

/// Instantiate the contract and use the voting module's token
/// contract as the proposal deposit token.
#[test]
fn test_voting_module_token_proposal_deposit_instantiate() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());

    let quorum = PercentageThreshold::Majority {};
    let voting_strategy = VotingStrategy::SingleChoice { quorum };
    let max_voting_period = cw_utils::Duration::Height(6);

    let instantiate = InstantiateMsg {
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        voting_strategy,
        pre_propose_info: get_pre_propose_info(
            &mut app,
            Some(UncheckedDepositInfo {
                denom: DepositToken::Token {
                    denom: UncheckedDenom::Native("ujuno".parse().unwrap()),
                },
                amount: Uint128::new(1),
                refund_policy: DepositRefundPolicy::OnlyPassed,
            }),
            false,
        ),
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);

    let gov_state: cwd_core::query::DumpStateResponse = app
        .wrap()
        .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
        .unwrap();
    let governance_modules = gov_state.proposal_modules;

    assert_eq!(governance_modules.len(), 1);
    let govmod = governance_modules.into_iter().next().unwrap().address;

    let (deposit_config, _) = query_deposit_config_and_pre_propose_module(&app, &govmod);
    assert_eq!(
        deposit_config.deposit_info,
        Some(CheckedDepositInfo {
            denom: CheckedDenom::Native("ujuno".parse().unwrap()),
            amount: Uint128::new(1),
            refund_policy: DepositRefundPolicy::OnlyPassed
        }),
    )
}

#[test]
fn test_native_proposal_deposit() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());

    let max_voting_period = cw_utils::Duration::Height(6);
    let instantiate = InstantiateMsg {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(100)),
        },
        max_voting_period,
        min_voting_period: None,
        only_members_execute: false,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: get_pre_propose_info(
            &mut app,
            Some(UncheckedDepositInfo {
                denom: DepositToken::Token {
                    denom: UncheckedDenom::Native("ujuno".to_string()),
                },
                amount: Uint128::new(1),
                refund_policy: DepositRefundPolicy::Always,
            }),
            false,
        ),
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        instantiate,
        Some(vec![Cw20Coin {
            address: "blue".to_string(),
            amount: Uint128::new(2),
        }]),
    );

    let gov_state: cwd_core::query::DumpStateResponse = app
        .wrap()
        .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
        .unwrap();
    let governance_modules = gov_state.proposal_modules;

    assert_eq!(governance_modules.len(), 1);
    let govmod = governance_modules.into_iter().next().unwrap().address;

    let (deposit_config, pre_propose_module) =
        query_deposit_config_and_pre_propose_module(&app, &govmod);
    if let CheckedDepositInfo {
        denom: CheckedDenom::Native(ref _token),
        refund_policy,
        ..
    } = deposit_config.deposit_info.unwrap()
    {
        assert_eq!(refund_policy, DepositRefundPolicy::Always);

        let mc_options = MultipleChoiceOptions {
            options: vec![
                MultipleChoiceOption {
                    description: "multiple choice option 1".to_string(),
                    msgs: None,
                },
                MultipleChoiceOption {
                    description: "multiple choice option 2".to_string(),
                    msgs: None,
                },
            ],
        };

        // This will fail because deposit not send
        app.execute_contract(
            Addr::unchecked("blue"),
            pre_propose_module.clone(),
            &cppm::ExecuteMsg::Propose {
                msg: cppm::ProposeMessage::Propose {
                    title: "title".to_string(),
                    description: "description".to_string(),
                    choices: mc_options.clone(),
                },
            },
            &[],
        )
        .unwrap_err();

        // Mint blue some tokens
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: "blue".to_string(),
            amount: vec![Coin {
                denom: "ujuno".to_string(),
                amount: Uint128::new(100),
            }],
        }))
        .unwrap();

        // Adding deposit will work
        make_proposal(&mut app, &govmod, "blue", mc_options);

        // "blue" has been refunded
        let balance = query_balance_native(&app, "blue", "ujuno");
        assert_eq!(balance, Uint128::new(99));

        // Govmod has refunded the token
        let balance = query_balance_native(&app, pre_propose_module.as_str(), "ujuno");
        assert_eq!(balance, Uint128::new(1));

        // Vote on the proposal.
        let res = app.execute_contract(
            Addr::unchecked("blue"),
            govmod.clone(),
            &ExecuteMsg::Vote {
                proposal_id: 1,
                vote: MultipleChoiceVote { option_id: 1 },
            },
            &[],
        );
        assert!(res.is_ok());

        // Execute the proposal, this should cause the deposit to be
        // refunded.
        app.execute_contract(
            Addr::unchecked("blue"),
            govmod.clone(),
            &ExecuteMsg::Execute { proposal_id: 1 },
            &[],
        )
        .unwrap();

        // "blue" has been refunded
        let balance = query_balance_native(&app, "blue", "ujuno");
        assert_eq!(balance, Uint128::new(100));

        // Govmod has refunded the token
        let balance = query_balance_native(&app, pre_propose_module.as_str(), "ujuno");
        assert_eq!(balance, Uint128::new(0));
    } else {
        panic!()
    };
}

#[test]
fn test_cant_propose_zero_power() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let quorum = PercentageThreshold::Percent(Decimal::percent(10));
    let voting_strategy = VotingStrategy::SingleChoice { quorum };
    let max_voting_period = Duration::Height(6);
    let instantiate = InstantiateMsg {
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        voting_strategy,
        pre_propose_info: get_pre_propose_info(
            &mut app,
            Some(UncheckedDepositInfo {
                denom: DepositToken::Token {
                    denom: UncheckedDenom::Native("ujuno".parse().unwrap()),
                },
                amount: Uint128::new(1),
                refund_policy: DepositRefundPolicy::OnlyPassed,
            }),
            false,
        ),
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        instantiate,
        Some(vec![
            Cw20Coin {
                address: "blue".to_string(),
                amount: Uint128::new(100000),
            },
            Cw20Coin {
                address: "blue2".to_string(),
                amount: Uint128::new(10000),
            },
        ]),
    );

    let gov_state: cwd_core::query::DumpStateResponse = app
        .wrap()
        .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
        .unwrap();
    let proposal_modules = gov_state.proposal_modules;

    assert_eq!(proposal_modules.len(), 1);
    let govmod = proposal_modules.into_iter().next().unwrap().address;

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];

    let mc_options = MultipleChoiceOptions { options };

    let (deposit_config, pre_propose_module) =
        query_deposit_config_and_pre_propose_module(&app, &govmod);
    if let Some(CheckedDepositInfo {
        denom: CheckedDenom::Cw20(ref token),
        amount,
        ..
    }) = deposit_config.deposit_info
    {
        app.execute_contract(
            Addr::unchecked("blue"),
            token.clone(),
            &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                spender: pre_propose_module.to_string(),
                amount,
                expires: None,
            },
            &[],
        )
        .unwrap();
    }

    mint_natives(&mut app, "blue", coins(10_000_000, "ujuno"));
    // Blue proposes
    app.execute_contract(
        Addr::unchecked("blue"),
        pre_propose_module.clone(),
        &cppm::ExecuteMsg::Propose {
            msg: cppm::ProposeMessage::Propose {
                title: "A simple text proposal".to_string(),
                description: "A simple text proposal".to_string(),
                choices: mc_options.clone(),
            },
        },
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::new(1),
        }],
    )
    .unwrap();

    // Should fail as blue's balance is now 0
    let err = app.execute_contract(
        Addr::unchecked("blue"),
        pre_propose_module,
        &cppm::ExecuteMsg::Propose {
            msg: cppm::ProposeMessage::Propose {
                title: "A simple text proposal".to_string(),
                description: "A simple text proposal".to_string(),
                choices: mc_options,
            },
        },
        &[],
    );

    assert!(err.is_err())
}

#[test]
fn test_open_proposal_submission() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());

    let max_voting_period = cw_utils::Duration::Height(6);

    // Instantiate with open_proposal_submission enabled
    let instantiate = InstantiateMsg {
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(100)),
        },
        max_voting_period,
        min_voting_period: None,
        only_members_execute: false,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: get_pre_propose_info(&mut app, None, true),
    };
    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let govmod = query_multiple_proposal_module(&app, &core_addr);

    make_proposal(
        &mut app,
        &govmod,
        "random",
        MultipleChoiceOptions {
            options: vec![
                MultipleChoiceOption {
                    description: "multiple choice option 1".to_string(),
                    msgs: None,
                },
                MultipleChoiceOption {
                    description: "multiple choice option 2".to_string(),
                    msgs: None,
                },
            ],
        },
    );

    let created: ProposalResponse = query_proposal(&app, &govmod, 1);
    let current_block = app.block_info();
    let expected = MultipleChoiceProposal {
        title: "title".to_string(),
        description: "description".to_string(),
        proposer: Addr::unchecked("random"),
        start_height: current_block.height,
        expiration: max_voting_period.after(&current_block),
        min_voting_period: None,
        allow_revoting: false,
        total_power: Uint128::new(100_000_000),
        status: Status::Open,
        voting_strategy: VotingStrategy::SingleChoice {
            quorum: PercentageThreshold::Percent(Decimal::percent(100)),
        },
        choices: vec![
            CheckedMultipleChoiceOption {
                description: "multiple choice option 1".to_string(),
                msgs: None,
                option_type: MultipleChoiceOptionType::Standard,
                vote_count: Uint128::zero(),
                index: 0,
            },
            CheckedMultipleChoiceOption {
                description: "multiple choice option 2".to_string(),
                msgs: None,
                option_type: MultipleChoiceOptionType::Standard,
                vote_count: Uint128::zero(),
                index: 1,
            },
            CheckedMultipleChoiceOption {
                description: "None of the above".to_string(),
                msgs: None,
                option_type: MultipleChoiceOptionType::None,
                vote_count: Uint128::zero(),
                index: 2,
            },
        ],
        votes: MultipleChoiceVotes {
            vote_weights: vec![Uint128::zero(); 3],
        },
    };

    assert_eq!(created.proposal, expected);
    assert_eq!(created.id, 1u64);
}

#[test]
fn test_execute_expired_proposal() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let quorum = PercentageThreshold::Percent(Decimal::percent(10));
    let voting_strategy = VotingStrategy::SingleChoice { quorum };
    let max_voting_period = cw_utils::Duration::Height(6);
    let instantiate = InstantiateMsg {
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        voting_strategy,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        instantiate,
        Some(vec![
            Cw20Coin {
                address: "blue".to_string(),
                amount: Uint128::new(10),
            },
            Cw20Coin {
                address: "inactive".to_string(),
                amount: Uint128::new(90),
            },
        ]),
    );

    let gov_state: cwd_core::query::DumpStateResponse = app
        .wrap()
        .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
        .unwrap();
    let proposal_modules = gov_state.proposal_modules;

    assert_eq!(proposal_modules.len(), 1);
    let govmod = proposal_modules.into_iter().next().unwrap().address;

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];

    let mc_options = MultipleChoiceOptions { options };

    app.execute_contract(
        Addr::unchecked("blue"),
        govmod.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("blue"),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    // Proposal has now reached quorum but should not be passed.
    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);
    assert_eq!(proposal.proposal.status, Status::Open);

    // Expire the proposal. It should now be passed as quorum was reached.
    app.update_block(|b| b.height += 10);

    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);
    assert_eq!(proposal.proposal.status, Status::Passed);

    // Try to close the proposal. This should fail as the proposal is
    // passed.
    app.execute_contract(
        Addr::unchecked("blue"),
        govmod.clone(),
        &ExecuteMsg::Close { proposal_id: 1 },
        &[],
    )
    .unwrap_err();

    // Check that we can execute the proposal despite the fact that it
    // is technically expired.
    app.execute_contract(
        Addr::unchecked("blue"),
        govmod.clone(),
        &ExecuteMsg::Execute { proposal_id: 1 },
        &[],
    )
    .unwrap();

    // Can't execute more than once.
    app.execute_contract(
        Addr::unchecked("blue"),
        govmod.clone(),
        &ExecuteMsg::Execute { proposal_id: 1 },
        &[],
    )
    .unwrap_err();

    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);
    assert_eq!(proposal.proposal.status, Status::Executed);
}

#[test]
fn test_query_list_proposals() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let quorum = PercentageThreshold::Majority {};
    let voting_strategy = VotingStrategy::SingleChoice { quorum };
    let max_voting_period = cw_utils::Duration::Height(6);
    let instantiate = InstantiateMsg {
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        voting_strategy: voting_strategy.clone(),
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };
    let gov_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        instantiate,
        Some(vec![Cw20Coin {
            address: CREATOR_ADDR.to_string(),
            amount: Uint128::new(100),
        }]),
    );

    let gov_modules: Vec<ProposalModule> = app
        .wrap()
        .query_wasm_smart(
            gov_addr,
            &cwd_core::msg::QueryMsg::ProposalModules {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(gov_modules.len(), 1);

    let govmod = gov_modules.into_iter().next().unwrap().address;

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];

    let mc_options = MultipleChoiceOptions { options };

    for _i in 1..10 {
        app.execute_contract(
            Addr::unchecked(CREATOR_ADDR),
            govmod.clone(),
            &ExecuteMsg::Propose {
                title: "A simple text proposal".to_string(),
                description: "A simple text proposal".to_string(),
                choices: mc_options.clone(),
                proposer: None,
            },
            &[],
        )
        .unwrap();
    }

    let proposals_forward: ProposalListResponse = query_list_proposals(&app, &govmod, None, None);
    let mut proposals_backward: ProposalListResponse =
        query_list_proposals_reverse(&app, &govmod, None, None);

    proposals_backward.proposals.reverse();

    assert_eq!(proposals_forward.proposals, proposals_backward.proposals);
    let checked_options = mc_options.into_checked().unwrap();
    let current_block = app.block_info();
    let expected = ProposalResponse {
        id: 1,
        proposal: MultipleChoiceProposal {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            proposer: Addr::unchecked(CREATOR_ADDR),
            start_height: current_block.height,
            expiration: max_voting_period.after(&current_block),
            choices: checked_options.options.clone(),
            status: Status::Open,
            voting_strategy: voting_strategy.clone(),
            total_power: Uint128::new(100),
            votes: MultipleChoiceVotes {
                vote_weights: vec![Uint128::zero(); 3],
            },
            allow_revoting: false,
            min_voting_period: None,
        },
    };
    assert_eq!(proposals_forward.proposals[0], expected);

    // Get proposals (3, 5]
    let proposals_forward: ProposalListResponse =
        query_list_proposals(&app, &govmod, Some(3), Some(2));

    let mut proposals_backward: ProposalListResponse =
        query_list_proposals_reverse(&app, &govmod, Some(6), Some(2));

    let expected = ProposalResponse {
        id: 4,
        proposal: MultipleChoiceProposal {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            proposer: Addr::unchecked(CREATOR_ADDR),
            start_height: current_block.height,
            expiration: max_voting_period.after(&current_block),
            choices: checked_options.options,
            status: Status::Open,
            voting_strategy,
            total_power: Uint128::new(100),
            votes: MultipleChoiceVotes {
                vote_weights: vec![Uint128::zero(); 3],
            },
            allow_revoting: false,
            min_voting_period: None,
        },
    };
    assert_eq!(proposals_forward.proposals[0], expected);
    assert_eq!(proposals_backward.proposals[1], expected);

    proposals_backward.proposals.reverse();
    assert_eq!(proposals_forward.proposals, proposals_backward.proposals);
}

#[test]
fn test_hooks() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());

    let quorum = PercentageThreshold::Majority {};
    let voting_strategy = VotingStrategy::SingleChoice { quorum };
    let max_voting_period = cw_utils::Duration::Height(6);
    let instantiate = InstantiateMsg {
        min_voting_period: None,
        close_proposal_on_execution_failure: true,
        max_voting_period,
        only_members_execute: false,
        allow_revoting: false,
        voting_strategy,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let govmod = query_multiple_proposal_module(&app, &core_addr);

    let govmod_config: Config = query_proposal_config(&app, &govmod);
    let dao = govmod_config.dao;

    // Expect no hooks
    let hooks: HooksResponse = query_proposal_hooks(&app, &govmod);
    assert_eq!(hooks.hooks.len(), 0);

    let hooks: HooksResponse = query_vote_hooks(&app, &govmod);
    assert_eq!(hooks.hooks.len(), 0);

    let msg = ExecuteMsg::AddProposalHook {
        address: "some_addr".to_string(),
    };

    // Expect error as sender is not DAO
    let _err = app
        .execute_contract(Addr::unchecked(CREATOR_ADDR), govmod.clone(), &msg, &[])
        .unwrap_err();

    // Expect success as sender is now DAO
    let _res = app
        .execute_contract(dao.clone(), govmod.clone(), &msg, &[])
        .unwrap();

    let hooks: HooksResponse = query_proposal_hooks(&app, &govmod);
    assert_eq!(hooks.hooks.len(), 1);

    // Expect error as hook is already set
    let _err = app
        .execute_contract(dao.clone(), govmod.clone(), &msg, &[])
        .unwrap_err();

    // Expect error as hook does not exist
    let _err = app
        .execute_contract(
            dao.clone(),
            govmod.clone(),
            &ExecuteMsg::RemoveProposalHook {
                address: "not_exist".to_string(),
            },
            &[],
        )
        .unwrap_err();

    let msg = ExecuteMsg::RemoveProposalHook {
        address: "some_addr".to_string(),
    };

    // Expect error as sender is not DAO
    let _err = app
        .execute_contract(Addr::unchecked(CREATOR_ADDR), govmod.clone(), &msg, &[])
        .unwrap_err();

    // Expect success
    let _res = app
        .execute_contract(dao.clone(), govmod.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::AddVoteHook {
        address: "some_addr".to_string(),
    };

    // Expect error as sender is not DAO
    let _err = app
        .execute_contract(Addr::unchecked(CREATOR_ADDR), govmod.clone(), &msg, &[])
        .unwrap_err();

    // Expect success as sender is now DAO
    let _res = app
        .execute_contract(dao.clone(), govmod.clone(), &msg, &[])
        .unwrap();

    let hooks: HooksResponse = query_vote_hooks(&app, &govmod);
    assert_eq!(hooks.hooks.len(), 1);

    // Expect error as hook is already set
    let _err = app
        .execute_contract(dao.clone(), govmod.clone(), &msg, &[])
        .unwrap_err();

    // Expect error as hook does not exist
    let _err = app
        .execute_contract(
            dao.clone(),
            govmod.clone(),
            &ExecuteMsg::RemoveVoteHook {
                address: "not_exist".to_string(),
            },
            &[],
        )
        .unwrap_err();

    let msg = ExecuteMsg::RemoveVoteHook {
        address: "some_addr".to_string(),
    };

    // Expect error as sender is not DAO
    let _err = app
        .execute_contract(Addr::unchecked(CREATOR_ADDR), govmod.clone(), &msg, &[])
        .unwrap_err();

    // Expect success
    let _res = app.execute_contract(dao, govmod, &msg, &[]).unwrap();
}

/// Basic test for revoting on prop-multiple
#[test]
fn test_revoting() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        InstantiateMsg {
            min_voting_period: None,
            max_voting_period: Duration::Height(6),
            only_members_execute: false,
            allow_revoting: true,
            voting_strategy: VotingStrategy::SingleChoice {
                quorum: PercentageThreshold::Majority {},
            },
            close_proposal_on_execution_failure: false,
            pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
        },
        Some(vec![
            Cw20Coin {
                address: "a-1".to_string(),
                amount: Uint128::new(100_000_000),
            },
            Cw20Coin {
                address: "a-2".to_string(),
                amount: Uint128::new(100_000_000),
            },
        ]),
    );

    let govmod = query_multiple_proposal_module(&app, &core_addr);

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];
    let mc_options = MultipleChoiceOptions { options };

    // Create a basic proposal with 2 options
    app.execute_contract(
        Addr::unchecked("a-1"),
        govmod.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // a-1 votes, vote_weights: [100_000_000, 0]
    app.execute_contract(
        Addr::unchecked("a-1"),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    // a-2 votes, vote_weights: [100_000_000, 100_000_000]
    app.execute_contract(
        Addr::unchecked("a-2"),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 1 },
        },
        &[],
    )
    .unwrap();

    // Time passes..
    app.update_block(|b| b.height += 2);

    // Assert that both vote options have equal vote weights at some block
    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);
    assert_eq!(proposal.proposal.status, Status::Open);
    assert_eq!(
        proposal.proposal.votes.vote_weights[0],
        Uint128::new(100_000_000),
    );
    assert_eq!(
        proposal.proposal.votes.vote_weights[1],
        Uint128::new(100_000_000),
    );

    // More time passes..
    app.update_block(|b| b.height += 3);

    // Last moment a-2 has a change of mind,
    // votes shift to [200_000_000, 0]
    app.execute_contract(
        Addr::unchecked("a-2"),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    app.update_block(next_block);

    // Assert that revote succeeded
    let proposal: ProposalResponse = query_proposal(&app, &govmod, 1);
    assert_eq!(proposal.proposal.status, Status::Passed);
    assert_eq!(
        proposal.proposal.votes.vote_weights[0],
        Uint128::new(200_000_000),
    );
    assert_eq!(proposal.proposal.votes.vote_weights[1], Uint128::new(0),);
}

/// Tests that revoting is stored at a per-proposal level.
/// Proposals created while revoting is enabled should not
/// have it disabled if a config change turns if off.
#[test]
fn test_allow_revoting_config_changes() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        InstantiateMsg {
            min_voting_period: None,
            max_voting_period: Duration::Height(6),
            only_members_execute: false,
            allow_revoting: true,
            voting_strategy: VotingStrategy::SingleChoice {
                quorum: PercentageThreshold::Majority {},
            },
            close_proposal_on_execution_failure: false,
            pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
        },
        Some(vec![
            Cw20Coin {
                address: "a-1".to_string(),
                amount: Uint128::new(100_000_000),
            },
            Cw20Coin {
                address: "a-2".to_string(),
                amount: Uint128::new(100_000_000),
            },
        ]),
    );

    let proposal_module = query_multiple_proposal_module(&app, &core_addr);

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];
    let mc_options = MultipleChoiceOptions { options };

    // Create a basic proposal with 2 options that allows revoting
    app.execute_contract(
        Addr::unchecked("a-1"),
        proposal_module.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            choices: mc_options.clone(),
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // Disable revoting
    app.execute_contract(
        core_addr.clone(),
        proposal_module.clone(),
        &ExecuteMsg::UpdateConfig {
            min_voting_period: None,
            max_voting_period: Duration::Height(6),
            only_members_execute: false,
            allow_revoting: false,
            dao: core_addr.to_string(),
            voting_strategy: VotingStrategy::SingleChoice {
                quorum: PercentageThreshold::Majority {},
            },
            close_proposal_on_execution_failure: false,
        },
        &[],
    )
    .unwrap();

    // Assert that proposal_id: 1 still allows revoting
    let proposal: ProposalResponse = query_proposal(&app, &proposal_module, 1);
    assert!(proposal.proposal.allow_revoting);

    app.execute_contract(
        Addr::unchecked("a-1"),
        proposal_module.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();
    app.execute_contract(
        Addr::unchecked("a-1"),
        proposal_module.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 1 },
        },
        &[],
    )
    .unwrap();

    // New proposals should not allow revoting
    app.execute_contract(
        Addr::unchecked("a-2"),
        proposal_module.clone(),
        &ExecuteMsg::Propose {
            title: "A very complex text proposal".to_string(),
            description: "A very complex text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("a-2"),
        proposal_module.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 2,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    let err: ContractError = app
        .execute_contract(
            Addr::unchecked("a-2"),
            proposal_module,
            &ExecuteMsg::Vote {
                proposal_id: 2,
                vote: MultipleChoiceVote { option_id: 1 },
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert!(matches!(err, ContractError::AlreadyVoted {}));
}

/// Tests that we error if a revote casts the same vote as the
/// previous vote.
#[test]
fn test_revoting_same_vote_twice() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        InstantiateMsg {
            min_voting_period: None,
            max_voting_period: Duration::Height(6),
            only_members_execute: false,
            allow_revoting: true,
            voting_strategy: VotingStrategy::SingleChoice {
                quorum: PercentageThreshold::Majority {},
            },
            close_proposal_on_execution_failure: false,
            pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
        },
        Some(vec![
            Cw20Coin {
                address: "a-1".to_string(),
                amount: Uint128::new(100_000_000),
            },
            Cw20Coin {
                address: "a-2".to_string(),
                amount: Uint128::new(100_000_000),
            },
        ]),
    );

    let proprosal_module = query_multiple_proposal_module(&app, &core_addr);

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];
    let mc_options = MultipleChoiceOptions { options };

    // Create a basic proposal with 2 options that allows revoting
    app.execute_contract(
        Addr::unchecked("a-1"),
        proprosal_module.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // Cast a vote
    app.execute_contract(
        Addr::unchecked("a-1"),
        proprosal_module.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    // Revote for the same option as currently voted
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked("a-1"),
            proprosal_module,
            &ExecuteMsg::Vote {
                proposal_id: 1,
                vote: MultipleChoiceVote { option_id: 0 },
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    // Can't cast the same vote twice.
    assert!(matches!(err, ContractError::AlreadyCast {}));
}

/// Tests that revoting into a non-existing vote option
/// does not invalidate the initial vote
#[test]
fn test_invalid_revote_does_not_invalidate_initial_vote() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());
    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        InstantiateMsg {
            min_voting_period: None,
            max_voting_period: Duration::Height(6),
            only_members_execute: false,
            allow_revoting: true,
            voting_strategy: VotingStrategy::SingleChoice {
                quorum: PercentageThreshold::Majority {},
            },
            close_proposal_on_execution_failure: false,
            pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
        },
        Some(vec![
            Cw20Coin {
                address: "a-1".to_string(),
                amount: Uint128::new(100_000_000),
            },
            Cw20Coin {
                address: "a-2".to_string(),
                amount: Uint128::new(100_000_000),
            },
        ]),
    );

    let proposal_module = query_multiple_proposal_module(&app, &core_addr);

    let options = vec![
        MultipleChoiceOption {
            description: "multiple choice option 1".to_string(),
            msgs: None,
        },
        MultipleChoiceOption {
            description: "multiple choice option 2".to_string(),
            msgs: None,
        },
    ];
    let mc_options = MultipleChoiceOptions { options };

    // Create a basic proposal with 2 options
    app.execute_contract(
        Addr::unchecked("a-1"),
        proposal_module.clone(),
        &ExecuteMsg::Propose {
            title: "A simple text proposal".to_string(),
            description: "A simple text proposal".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // a-1 votes, vote_weights: [100_000_000, 0]
    app.execute_contract(
        Addr::unchecked("a-1"),
        proposal_module.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    // a-2 votes, vote_weights: [100_000_000, 100_000_000]
    app.execute_contract(
        Addr::unchecked("a-2"),
        proposal_module.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 1 },
        },
        &[],
    )
    .unwrap();

    app.update_block(next_block);

    // Assert that both vote options have equal vote weights at some block
    let proposal: ProposalResponse = query_proposal(&app, &proposal_module, 1);
    assert_eq!(proposal.proposal.status, Status::Open);
    assert_eq!(
        proposal.proposal.votes.vote_weights[0],
        Uint128::new(100_000_000),
    );
    assert_eq!(
        proposal.proposal.votes.vote_weights[1],
        Uint128::new(100_000_000),
    );

    // Time passes..
    app.update_block(|b| b.height += 3);

    // Last moment a-2 has a change of mind and attempts
    // to vote for a non-existing option
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked("a-2"),
            proposal_module,
            &ExecuteMsg::Vote {
                proposal_id: 1,
                vote: MultipleChoiceVote { option_id: 99 },
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    // Assert that prior votes remained the same
    assert_eq!(
        proposal.proposal.votes.vote_weights[0],
        Uint128::new(100_000_000),
    );
    assert_eq!(
        proposal.proposal.votes.vote_weights[1],
        Uint128::new(100_000_000),
    );
    assert!(matches!(err, ContractError::InvalidVote {}));
}

#[test]
fn test_close_failed_proposal() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let _govmod_id = app.store_code(proposal_multiple_contract());

    let quorum = PercentageThreshold::Majority {};
    let voting_strategy = VotingStrategy::SingleChoice { quorum };
    let max_voting_period = cw_utils::Duration::Height(6);
    let instantiate = InstantiateMsg {
        max_voting_period,
        voting_strategy,
        min_voting_period: None,
        only_members_execute: false,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
    };

    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let govmod = query_multiple_proposal_module(&app, &core_addr);

    let msg = cw20::Cw20ExecuteMsg::Burn {
        amount: Uint128::new(2000),
    };
    let binary_msg = to_binary(&msg).unwrap();

    let options = vec![
        MultipleChoiceOption {
            description: "Burn or burn".to_string(),
            msgs: Some(vec![WasmMsg::Execute {
                contract_addr: "token_contract".to_string(),
                msg: binary_msg,
                funds: vec![],
            }
            .into()]),
        },
        MultipleChoiceOption {
            description: "Don't burn".to_string(),
            msgs: None,
        },
    ];

    let mc_options = MultipleChoiceOptions { options };

    // Overburn tokens
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod.clone(),
        &ExecuteMsg::Propose {
            title: "A simple burn tokens proposal".to_string(),
            description: "Burning more tokens, than dao reserve have".to_string(),
            choices: mc_options.clone(),
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // Vote on proposal
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    // Update block
    let timestamp = Timestamp::from_seconds(300_000_000);
    app.update_block(|block| block.time = timestamp);

    // Execute proposal
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod.clone(),
        &ExecuteMsg::Execute { proposal_id: 1 },
        &[],
    )
    .unwrap();

    let failed: ProposalResponse = query_proposal(&app, &govmod, 1);

    assert_eq!(failed.proposal.status, Status::ExecutionFailed);
    // With disabled feature
    // Disable feature first
    {
        let original: Config = query_proposal_config(&app, &govmod);

        app.execute_contract(
            Addr::unchecked(CREATOR_ADDR),
            govmod.clone(),
            &ExecuteMsg::Propose {
                title: "Disable closing failed proposals".to_string(),
                description: "We want to re-execute failed proposals".to_string(),
                choices: MultipleChoiceOptions {
                    options: vec![
                        MultipleChoiceOption {
                            description: "Disable closing failed proposals".to_string(),
                            msgs: Some(vec![WasmMsg::Execute {
                                contract_addr: govmod.to_string(),
                                msg: to_binary(&ExecuteMsg::UpdateConfig {
                                    voting_strategy: VotingStrategy::SingleChoice { quorum },
                                    max_voting_period: original.max_voting_period,
                                    min_voting_period: original.min_voting_period,
                                    only_members_execute: original.only_members_execute,
                                    allow_revoting: false,
                                    dao: original.dao.to_string(),
                                    close_proposal_on_execution_failure: false,
                                })
                                .unwrap(),
                                funds: vec![],
                            }
                            .into()]),
                        },
                        MultipleChoiceOption {
                            description: "Don't disable".to_string(),
                            msgs: None,
                        },
                    ],
                },
                proposer: None,
            },
            &[],
        )
        .unwrap();

        // Vote on proposal
        app.execute_contract(
            Addr::unchecked(CREATOR_ADDR),
            govmod.clone(),
            &ExecuteMsg::Vote {
                proposal_id: 2,
                vote: MultipleChoiceVote { option_id: 0 },
            },
            &[],
        )
        .unwrap();

        // Execute proposal
        app.execute_contract(
            Addr::unchecked(CREATOR_ADDR),
            govmod.clone(),
            &ExecuteMsg::Execute { proposal_id: 2 },
            &[],
        )
        .unwrap();
    }

    // Overburn tokens (again), this time without reverting
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod.clone(),
        &ExecuteMsg::Propose {
            title: "A simple burn tokens proposal".to_string(),
            description: "Burning more tokens, than dao reserve have".to_string(),
            choices: mc_options,
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // Vote on proposal
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod.clone(),
        &ExecuteMsg::Vote {
            proposal_id: 3,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    // Execute proposal
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod.clone(),
        &ExecuteMsg::Execute { proposal_id: 3 },
        &[],
    )
    .expect_err("Should be sub overflow");

    // Status should still be passed
    let updated: ProposalResponse = query_proposal(&app, &govmod, 3);

    // not reverted
    assert_eq!(updated.proposal.status, Status::Passed);
}
