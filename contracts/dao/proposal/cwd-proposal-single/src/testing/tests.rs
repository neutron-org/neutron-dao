use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_env},
    to_binary, Addr, Attribute, BankMsg, ContractInfoResponse, CosmosMsg, Decimal, Empty, Reply,
    StdError, StdResult, SubMsgResult, Uint128, WasmMsg, WasmQuery,
};
use cosmwasm_std::{Api, Storage};
use cw2::ContractVersion;
use cw20::Cw20Coin;
use cw_multi_test::{custom_app, BasicApp, Executor, Router};
use cw_utils::Duration;
use cwd_core::msg::{ExecuteMsg as DaoExecuteMsg, QueryMsg as DaoQueryMsg};
use cwd_core::query::SubDao;
use cwd_hooks::{HookError, HooksResponse};
use cwd_interface::voting::InfoResponse;
use cwd_voting::{
    pre_propose::{PreProposeInfo, ProposalCreationPolicy},
    proposal::MAX_PROPOSAL_SIZE,
    reply::{
        failed_pre_propose_module_hook_id, mask_proposal_execution_proposal_id,
        mask_proposal_hook_index, mask_vote_hook_index,
    },
    status::Status,
    threshold::{PercentageThreshold, Threshold},
    voting::{Vote, Votes},
};
use neutron_sdk::bindings::msg::NeutronMsg;

use crate::testing::execute::{execute_proposal, execute_proposal_should_fail};
use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    msg::{ExecuteMsg, QueryMsg},
    proposal::SingleChoiceProposal,
    query::{ProposalResponse, VoteInfo},
    state::Config,
    testing::{
        execute::{
            add_proposal_hook, add_proposal_hook_should_fail, add_vote_hook,
            add_vote_hook_should_fail, close_proposal, close_proposal_should_fail, make_proposal,
            mint_natives, remove_proposal_hook, remove_proposal_hook_should_fail, remove_vote_hook,
            remove_vote_hook_should_fail, vote_on_proposal, vote_on_proposal_should_fail,
        },
        instantiate::{
            get_proposal_module_instantiate, instantiate_with_native_bonded_balances_governance,
        },
        queries::{
            query_balance_native, query_creation_policy, query_list_proposals,
            query_list_proposals_reverse, query_list_votes, query_proposal, query_proposal_config,
            query_proposal_hooks, query_single_proposal_module, query_vote_hooks,
        },
    },
    ContractError,
};

use super::CREATOR_ADDR;

struct CommonTest {
    app: BasicApp<NeutronMsg>,
    core_addr: Addr,
    proposal_module: Addr,
    proposal_id: u64,
}

pub(crate) fn no_init<BankT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT>(
    _: &mut Router<BankT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT>,
    _: &dyn Api,
    _: &mut dyn Storage,
) {
}

fn setup_test(messages: Vec<CosmosMsg<NeutronMsg>>) -> CommonTest {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let instantiate = get_proposal_module_instantiate(&mut app);
    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let proposal_module = query_single_proposal_module(&app, &core_addr);

    // Mint some tokens to pay the proposal deposit.
    mint_natives(&mut app, CREATOR_ADDR, coins(10_000_000, "ujuno"));
    let proposal_id = make_proposal(&mut app, &proposal_module, CREATOR_ADDR, messages);

    CommonTest {
        app,
        core_addr,
        proposal_module,
        proposal_id,
    }
}

#[test]
fn test_proposal_message_execution() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let mut instantiate = get_proposal_module_instantiate(&mut app);
    instantiate.close_proposal_on_execution_failure = false;
    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let proposal_module = query_single_proposal_module(&app, &core_addr);

    mint_natives(&mut app, CREATOR_ADDR, coins(10000000, "ujuno"));
    let proposal_id = make_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        vec![BankMsg::Send {
            to_address: CREATOR_ADDR.to_string(),
            amount: coins(1, "ujuno"),
        }
        .into()],
    );
    let native_balance = query_balance_native(&app, CREATOR_ADDR, "ujuno");
    assert_eq!(native_balance, Uint128::zero());

    vote_on_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        proposal_id,
        Vote::Yes,
    );
    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.status, Status::Passed);

    // Can't use library function because we expect this to fail due
    // to insufficent balance in the bank module.
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        proposal_module.clone(),
        &ExecuteMsg::Execute { proposal_id },
        &[],
    )
    .unwrap_err();
    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.status, Status::Passed);

    mint_natives(&mut app, core_addr.as_str(), coins(10, "ujuno"));
    execute_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.status, Status::Executed);

    let native_balance = query_balance_native(&app, CREATOR_ADDR, "ujuno");
    assert_eq!(native_balance, Uint128::new(10000001));

    // Sneak in a check here that proposals can't be executed more
    // than once in the on close on execute config suituation.
    let err = execute_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::NotPassed {}))
}

#[test]
fn test_proposal_close_after_expiry() {
    let CommonTest {
        mut app,
        core_addr,
        proposal_module,
        proposal_id,
    } = setup_test(vec![BankMsg::Send {
        to_address: CREATOR_ADDR.to_string(),
        amount: coins(10, "ujuno"),
    }
    .into()]);
    mint_natives(&mut app, core_addr.as_str(), coins(10, "ujuno"));

    // Try and close the proposal. This shoudl fail as the proposal is
    // open.
    let err = close_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::WrongCloseStatus {}));

    // Expire the proposal. Now it should be closable.
    app.update_block(|mut b| b.time = b.time.plus_seconds(604800));
    close_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.status, Status::Closed);
}

#[test]
fn test_proposal_cant_close_after_expiry_is_passed() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let instantiate = get_proposal_module_instantiate(&mut app);
    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        instantiate,
        Some(vec![
            Cw20Coin {
                address: "quorum".to_string(),
                amount: Uint128::new(15),
            },
            Cw20Coin {
                address: CREATOR_ADDR.to_string(),
                amount: Uint128::new(85),
            },
        ]),
    );
    let proposal_module = query_single_proposal_module(&app, &core_addr);

    mint_natives(&mut app, core_addr.as_str(), coins(10, "ujuno"));
    mint_natives(&mut app, CREATOR_ADDR, coins(10000000, "ujuno"));
    let proposal_id = make_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        vec![BankMsg::Send {
            to_address: CREATOR_ADDR.to_string(),
            amount: coins(1, "ujuno"),
        }
        .into()],
    );
    vote_on_proposal(&mut app, &proposal_module, "quorum", proposal_id, Vote::Yes);
    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.status, Status::Open);

    // Expire the proposal. This should pass it.
    app.update_block(|mut b| b.time = b.time.plus_seconds(604800));
    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.status, Status::Passed);

    // Make sure it can't be closed.
    let err = close_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::WrongCloseStatus {}));

    // Executed proposals may not be closed.
    execute_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    let err = close_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::WrongCloseStatus {}));
    let balance = query_balance_native(&app, CREATOR_ADDR, "ujuno");
    assert_eq!(balance, Uint128::new(10000001));
    let err = close_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::WrongCloseStatus {}));
}

#[test]
fn test_execute_no_non_passed_execution() {
    let CommonTest {
        mut app,
        core_addr,
        proposal_module,
        proposal_id,
    } = setup_test(vec![BankMsg::Send {
        to_address: CREATOR_ADDR.to_string(),
        amount: coins(10, "ujuno"),
    }
    .into()]);
    mint_natives(&mut app, core_addr.as_str(), coins(100, "ujuno"));

    let err = execute_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::NotPassed {}));

    // Expire the proposal.
    app.update_block(|mut b| b.time = b.time.plus_seconds(604800));
    let err = execute_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::NotPassed {}));

    mint_natives(&mut app, CREATOR_ADDR, coins(10000000, "ujuno"));
    let proposal_id = make_proposal(&mut app, &proposal_module, CREATOR_ADDR, vec![]);
    vote_on_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        proposal_id,
        Vote::Yes,
    );
    execute_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    // Can't execute more than once.
    let err = execute_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::NotPassed {}));
}

#[test]
fn test_update_config() {
    let CommonTest {
        mut app,
        core_addr,
        proposal_module,
        proposal_id,
    } = setup_test(vec![]);
    vote_on_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        proposal_id,
        Vote::Yes,
    );
    execute_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    // Make a proposal to update the config.
    let proposal_id = make_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        vec![WasmMsg::Execute {
            contract_addr: proposal_module.to_string(),
            msg: to_binary(&ExecuteMsg::UpdateConfig {
                threshold: Threshold::AbsoluteCount {
                    threshold: Uint128::new(10_000),
                },
                max_voting_period: Duration::Height(6),
                min_voting_period: None,
                allow_revoting: false,
                dao: core_addr.to_string(),
                close_proposal_on_execution_failure: false,
            })
            .unwrap(),
            funds: vec![],
        }
        .into()],
    );
    vote_on_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        proposal_id,
        Vote::Yes,
    );
    execute_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);

    let config = query_proposal_config(&app, &proposal_module);
    assert_eq!(
        config,
        Config {
            threshold: Threshold::AbsoluteCount {
                threshold: Uint128::new(10_000)
            },
            max_voting_period: Duration::Height(6),
            min_voting_period: None,
            allow_revoting: false,
            dao: core_addr.clone(),
            close_proposal_on_execution_failure: false,
        }
    );

    // Check that non-dao address may not update config.
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(CREATOR_ADDR),
            proposal_module,
            &&ExecuteMsg::UpdateConfig {
                threshold: Threshold::AbsoluteCount {
                    threshold: Uint128::new(10_000),
                },
                max_voting_period: Duration::Height(6),
                min_voting_period: None,
                allow_revoting: false,
                dao: core_addr.to_string(),
                close_proposal_on_execution_failure: false,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert!(matches!(err, ContractError::Unauthorized {}))
}

#[test]
fn test_anyone_may_propose_and_proposal_listing() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let mut instantiate = get_proposal_module_instantiate(&mut app);
    instantiate.pre_propose_info = PreProposeInfo::AnyoneMayPropose {};
    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let proposal_module = query_single_proposal_module(&app, &core_addr);

    for addr in 'm'..'z' {
        let addr = addr.to_string().repeat(6);
        let proposal_id = make_proposal(&mut app, &proposal_module, &addr, vec![]);
        vote_on_proposal(
            &mut app,
            &proposal_module,
            CREATOR_ADDR,
            proposal_id,
            Vote::Yes,
        );
        execute_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    }

    // Now that we've got all these proposals sitting around, lets
    // test that we can query them.

    let proposals_forward = query_list_proposals(&app, &proposal_module, None, None);
    let mut proposals_reverse = query_list_proposals_reverse(&app, &proposal_module, None, None);
    proposals_reverse.proposals.reverse();
    assert_eq!(proposals_reverse, proposals_forward);

    // Check the proposers and (implicitly) the ordering.
    for (index, addr) in ('m'..'z').enumerate() {
        let addr = addr.to_string().repeat(6);
        assert_eq!(
            proposals_forward.proposals[index].proposal.proposer,
            Addr::unchecked(addr)
        )
    }

    let four_and_five = query_list_proposals(&app, &proposal_module, Some(3), Some(2));
    let mut five_and_four = query_list_proposals_reverse(&app, &proposal_module, Some(6), Some(2));
    five_and_four.proposals.reverse();

    assert_eq!(five_and_four, four_and_five);
    assert_eq!(
        four_and_five.proposals[0].proposal.proposer,
        Addr::unchecked("pppppp")
    );

    let current_block = app.block_info();
    assert_eq!(
        four_and_five.proposals[0],
        ProposalResponse {
            id: 4,
            proposal: SingleChoiceProposal {
                title: "title".to_string(),
                description: "description".to_string(),
                proposer: Addr::unchecked("pppppp"),
                start_height: current_block.height,
                min_voting_period: None,
                expiration: Duration::Time(604800).after(&current_block),
                threshold: Threshold::ThresholdQuorum {
                    quorum: PercentageThreshold::Percent(Decimal::percent(15)),
                    threshold: PercentageThreshold::Majority {},
                },
                allow_revoting: false,
                total_power: Uint128::new(100_000_000),
                msgs: vec![],
                status: Status::Executed,
                votes: Votes {
                    yes: Uint128::new(100_000_000),
                    no: Uint128::zero(),
                    abstain: Uint128::zero()
                },
            }
        }
    )
}

#[test]
fn test_proposal_hook_registration() {
    let CommonTest {
        mut app,
        core_addr,
        proposal_module,
        proposal_id: _,
    } = setup_test(vec![]);

    let proposal_hooks = query_proposal_hooks(&app, &proposal_module);
    assert_eq!(
        proposal_hooks.hooks.len(),
        0,
        "pre-propose deposit module should not show on this listing"
    );

    // non-dao may not add a hook.
    let err =
        add_proposal_hook_should_fail(&mut app, &proposal_module, CREATOR_ADDR, "proposalhook");
    assert!(matches!(err, ContractError::Unauthorized {}));

    add_proposal_hook(
        &mut app,
        &proposal_module,
        core_addr.as_str(),
        "proposalhook",
    );
    let err = add_proposal_hook_should_fail(
        &mut app,
        &proposal_module,
        core_addr.as_str(),
        "proposalhook",
    );
    assert!(matches!(
        err,
        ContractError::HookError(HookError::HookAlreadyRegistered {})
    ));

    let proposal_hooks = query_proposal_hooks(&app, &proposal_module);
    assert_eq!(proposal_hooks.hooks[0], "proposalhook".to_string());

    // Only DAO can remove proposal hooks.
    let err =
        remove_proposal_hook_should_fail(&mut app, &proposal_module, CREATOR_ADDR, "proposalhook");
    assert!(matches!(err, ContractError::Unauthorized {}));
    remove_proposal_hook(
        &mut app,
        &proposal_module,
        core_addr.as_str(),
        "proposalhook",
    );
    let proposal_hooks = query_proposal_hooks(&app, &proposal_module);
    assert_eq!(proposal_hooks.hooks.len(), 0);

    // Can not remove that which does not exist.
    let err = remove_proposal_hook_should_fail(
        &mut app,
        &proposal_module,
        core_addr.as_str(),
        "proposalhook",
    );
    assert!(matches!(
        err,
        ContractError::HookError(HookError::HookNotRegistered {})
    ));
}

#[test]
fn test_vote_hook_registration() {
    let CommonTest {
        mut app,
        core_addr,
        proposal_module,
        proposal_id: _,
    } = setup_test(vec![]);

    let vote_hooks = query_vote_hooks(&app, &proposal_module);
    assert!(vote_hooks.hooks.is_empty(),);

    // non-dao may not add a hook.
    let err = add_vote_hook_should_fail(&mut app, &proposal_module, CREATOR_ADDR, "votehook");
    assert!(matches!(err, ContractError::Unauthorized {}));

    add_vote_hook(&mut app, &proposal_module, core_addr.as_str(), "votehook");

    let vote_hooks = query_vote_hooks(&app, &proposal_module);
    assert_eq!(
        vote_hooks,
        HooksResponse {
            hooks: vec!["votehook".to_string()]
        }
    );

    let err = add_vote_hook_should_fail(&mut app, &proposal_module, core_addr.as_str(), "votehook");
    assert!(matches!(
        err,
        ContractError::HookError(HookError::HookAlreadyRegistered {})
    ));

    let vote_hooks = query_vote_hooks(&app, &proposal_module);
    assert_eq!(vote_hooks.hooks[0], "votehook".to_string());

    // Only DAO can remove vote hooks.
    let err = remove_vote_hook_should_fail(&mut app, &proposal_module, CREATOR_ADDR, "votehook");
    assert!(matches!(err, ContractError::Unauthorized {}));
    remove_vote_hook(&mut app, &proposal_module, core_addr.as_str(), "votehook");

    let vote_hooks = query_vote_hooks(&app, &proposal_module);
    assert!(vote_hooks.hooks.is_empty(),);

    // Can not remove that which does not exist.
    let err =
        remove_vote_hook_should_fail(&mut app, &proposal_module, core_addr.as_str(), "votehook");
    assert!(matches!(
        err,
        ContractError::HookError(HookError::HookNotRegistered {})
    ));
}

#[test]
#[should_panic(
    expected = "min_voting_period and max_voting_period must have the same units (height or time)"
)]
fn test_min_duration_unit_missmatch() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let mut instantiate = get_proposal_module_instantiate(&mut app);
    instantiate.min_voting_period = Some(Duration::Height(10));
    instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
}

#[test]
#[should_panic(expected = "Min voting period must be less than or equal to max voting period")]
fn test_min_duration_larger_than_proposal_duration() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let mut instantiate = get_proposal_module_instantiate(&mut app);
    instantiate.min_voting_period = Some(Duration::Time(604801));
    instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
}

#[test]
fn test_min_voting_period_no_early_pass() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let mut instantiate = get_proposal_module_instantiate(&mut app);
    instantiate.min_voting_period = Some(Duration::Height(10));
    instantiate.max_voting_period = Duration::Height(100);
    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let proposal_module = query_single_proposal_module(&app, &core_addr);

    mint_natives(&mut app, CREATOR_ADDR, coins(10_000_000, "ujuno"));
    let proposal_id = make_proposal(&mut app, &proposal_module, CREATOR_ADDR, vec![]);
    vote_on_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        proposal_id,
        Vote::Yes,
    );
    let proposal_response = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal_response.proposal.status, Status::Open);

    app.update_block(|mut block| block.height += 10);
    let proposal_response = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal_response.proposal.status, Status::Passed);
}

// Setting the min duration the same as the proposal duration just
// means that proposals cant close early.
#[test]
fn test_min_duration_same_as_proposal_duration() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let mut instantiate = get_proposal_module_instantiate(&mut app);
    instantiate.min_voting_period = Some(Duration::Height(100));
    instantiate.max_voting_period = Duration::Height(100);
    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        instantiate,
        Some(vec![
            Cw20Coin {
                address: "ekez".to_string(),
                amount: Uint128::new(10),
            },
            Cw20Coin {
                address: "whale".to_string(),
                amount: Uint128::new(90),
            },
        ]),
    );
    let proposal_module = query_single_proposal_module(&app, &core_addr);

    mint_natives(&mut app, "ekez", coins(10_000_000, "ujuno"));
    let proposal_id = make_proposal(&mut app, &proposal_module, "ekez", vec![]);

    // Whale votes yes. Normally the proposal would just pass and ekez
    // would be out of luck.
    vote_on_proposal(&mut app, &proposal_module, "whale", proposal_id, Vote::Yes);
    vote_on_proposal(&mut app, &proposal_module, "ekez", proposal_id, Vote::No);

    app.update_block(|mut b| b.height += 100);
    let proposal_response = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal_response.proposal.status, Status::Passed);
}

// #[test]
// fn test_revoting_playthrough() {
//     let mut app = App::default();
//     let mut instantiate = get_default_token_dao_proposal_module_instantiate(&mut app);
//     instantiate.allow_revoting = true;
//     let core_addr = instantiate_with_native_staked_balances_governance(&mut app, instantiate, None);
//     let gov_token = query_dao_token(&app, &core_addr);
//     let proposal_module = query_single_proposal_module(&app, &core_addr);
//
//     mint_cw20s(&mut app, &gov_token, &core_addr, CREATOR_ADDR, 10_000_000);
//     let proposal_id = make_proposal(&mut app, &proposal_module, CREATOR_ADDR, vec![]);
//
//     // Vote and change our minds a couple times.
//     vote_on_proposal(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         proposal_id,
//         Vote::Yes,
//     );
//     let proposal_response = query_proposal(&app, &proposal_module, proposal_id);
//     assert_eq!(proposal_response.proposal.status, Status::Open);
//
//     vote_on_proposal(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         proposal_id,
//         Vote::No,
//     );
//     let proposal_response = query_proposal(&app, &proposal_module, proposal_id);
//     assert_eq!(proposal_response.proposal.status, Status::Open);
//
//     vote_on_proposal(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         proposal_id,
//         Vote::Yes,
//     );
//     let proposal_response = query_proposal(&app, &proposal_module, proposal_id);
//     assert_eq!(proposal_response.proposal.status, Status::Open);
//
//     // Can't cast the same vote more than once.
//     let err = vote_on_proposal_should_fail(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         proposal_id,
//         Vote::Yes,
//     );
//     assert!(matches!(err, ContractError::AlreadyCast {}));
//
//     // Expire the proposal allowing the votes to be tallied.
//     app.update_block(|mut b| b.time = b.time.plus_seconds(604800));
//     let proposal_response = query_proposal(&app, &proposal_module, proposal_id);
//     assert_eq!(proposal_response.proposal.status, Status::Passed);
//     execute_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
//
//     // Can't vote once the proposal is passed.
//     let err = vote_on_proposal_should_fail(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         proposal_id,
//         Vote::Yes,
//     );
//     assert!(matches!(err, ContractError::NotOpen { .. }));
// }
//
// /// Tests that revoting is stored at a per-proposal level. Proposals
// /// created while revoting is enabled should not have it disabled if a
// /// config change turns if off.
// #[test]
// fn test_allow_revoting_config_changes() {
//     let mut app = App::default();
//     let mut instantiate = get_default_token_dao_proposal_module_instantiate(&mut app);
//     instantiate.allow_revoting = true;
//     let core_addr = instantiate_with_native_staked_balances_governance(&mut app, instantiate, None);
//     let gov_token = query_dao_token(&app, &core_addr);
//     let proposal_module = query_single_proposal_module(&app, &core_addr);
//
//     mint_cw20s(&mut app, &gov_token, &core_addr, CREATOR_ADDR, 10_000_000);
//     // This proposal should have revoting enable for its entire
//     // lifetime.
//     let revoting_proposal = make_proposal(&mut app, &proposal_module, CREATOR_ADDR, vec![]);
//
//     // Update the config of the proposal module to disable revoting.
//     app.execute_contract(
//         core_addr.clone(),
//         proposal_module.clone(),
//         &ExecuteMsg::UpdateConfig {
//             threshold: Threshold::ThresholdQuorum {
//                 quorum: PercentageThreshold::Percent(Decimal::percent(15)),
//                 threshold: PercentageThreshold::Majority {},
//             },
//             max_voting_period: Duration::Height(10),
//             min_voting_period: None,
//             // Turn off revoting.
//             allow_revoting: false,
//             dao: core_addr.to_string(),
//             close_proposal_on_execution_failure: false,
//         },
//         &[],
//     )
//     .unwrap();
//
//     mint_cw20s(&mut app, &gov_token, &core_addr, CREATOR_ADDR, 10_000_000);
//     let no_revoting_proposal = make_proposal(&mut app, &proposal_module, CREATOR_ADDR, vec![]);
//
//     vote_on_proposal(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         revoting_proposal,
//         Vote::Yes,
//     );
//     vote_on_proposal(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         no_revoting_proposal,
//         Vote::Yes,
//     );
//
//     // Proposal without revoting should have passed.
//     let proposal_resp = query_proposal(&app, &proposal_module, no_revoting_proposal);
//     assert_eq!(proposal_resp.proposal.status, Status::Passed);
//
//     // Proposal with revoting should not have passed.
//     let proposal_resp = query_proposal(&app, &proposal_module, revoting_proposal);
//     assert_eq!(proposal_resp.proposal.status, Status::Open);
//
//     // Can not vote again on the no revoting proposal.
//     let err = vote_on_proposal_should_fail(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         no_revoting_proposal,
//         Vote::No,
//     );
//     assert!(matches!(err, ContractError::NotOpen { .. }));
//
//     // Can change vote on the revoting proposal.
//     vote_on_proposal(
//         &mut app,
//         &proposal_module,
//         CREATOR_ADDR,
//         revoting_proposal,
//         Vote::No,
//     );
//     // Expire the revoting proposal and close it.
//     app.update_block(|mut b| b.time = b.time.plus_seconds(604800));
//     close_proposal(&mut app, &proposal_module, CREATOR_ADDR, revoting_proposal);
// }
//
// #[test]
// fn test_proposal_count_initialized_to_zero() {
//     let mut app = App::default();
//     let pre_propose_info = get_pre_propose_info(&mut app, None, false);
//     let core_addr = instantiate_with_native_staked_balances_governance(
//         &mut app,
//         InstantiateMsg {
//             threshold: Threshold::ThresholdQuorum {
//                 threshold: PercentageThreshold::Majority {},
//                 quorum: PercentageThreshold::Percent(Decimal::percent(10)),
//             },
//             max_voting_period: Duration::Height(10),
//             min_voting_period: None,
//             allow_revoting: false,
//             pre_propose_info,
//             close_proposal_on_execution_failure: true,
//         },
//         Some(vec![
//             Cw20Coin {
//                 address: "ekez".to_string(),
//                 amount: Uint128::new(10),
//             },
//             Cw20Coin {
//                 address: "innactive".to_string(),
//                 amount: Uint128::new(90),
//             },
//         ]),
//     );
//
//     let core_state: cwd_core::query::DumpStateResponse = app
//         .wrap()
//         .query_wasm_smart(core_addr, &cwd_core::msg::QueryMsg::DumpState {})
//         .unwrap();
//     let proposal_modules = core_state.proposal_modules;
//
//     assert_eq!(proposal_modules.len(), 1);
//     let proposal_single = proposal_modules.into_iter().next().unwrap().address;
//
//     let proposal_count: u64 = app
//         .wrap()
//         .query_wasm_smart(proposal_single, &QueryMsg::ProposalCount {})
//         .unwrap();
//     assert_eq!(proposal_count, 0);
// }

// - Make a proposal that will fail to execute.
// - Verify that it goes to execution failed and that proposal
//   deposits are returned once and not on closing.
// - Make the same proposal again.
// - Update the config to disable close on execution failure.
// - Make sure that proposal doesn't close on execution (this config
//   feature gets applied retroactively).
#[test]
fn test_execution_failed() {
    let CommonTest {
        mut app,
        core_addr,
        proposal_module,
        proposal_id,
    } = setup_test(vec![BankMsg::Send {
        to_address: "ekez".to_string(),
        amount: coins(10, "ujuno"),
    }
    .into()]);

    vote_on_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        proposal_id,
        Vote::Yes,
    );
    execute_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);

    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.status, Status::ExecutionFailed);

    // Make sure the deposit was returned.
    let balance = query_balance_native(&app, CREATOR_ADDR, "ujuno");
    assert_eq!(balance, Uint128::new(10_000_000));

    // ExecutionFailed is an end state.
    let err = close_proposal_should_fail(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
    assert!(matches!(err, ContractError::WrongCloseStatus {}));

    let proposal_id = make_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        vec![BankMsg::Send {
            to_address: "ekez".to_string(),
            amount: coins(10, "ujuno"),
        }
        .into()],
    );

    let config = query_proposal_config(&app, &proposal_module);

    // Disable execution failing proposals.
    app.execute_contract(
        core_addr,
        proposal_module.clone(),
        &ExecuteMsg::UpdateConfig {
            threshold: config.threshold,
            max_voting_period: config.max_voting_period,
            min_voting_period: config.min_voting_period,
            allow_revoting: config.allow_revoting,
            dao: config.dao.into_string(),
            // Disable.
            close_proposal_on_execution_failure: false,
        },
        &[],
    )
    .unwrap();

    vote_on_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        proposal_id,
        Vote::Yes,
    );
    let err: StdError = app
        .execute_contract(
            Addr::unchecked(CREATOR_ADDR),
            proposal_module.clone(),
            &ExecuteMsg::Execute { proposal_id },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert!(matches!(err, StdError::Overflow { .. }));

    // Even though this proposal was created before the config change
    // was made it still gets retroactively applied.
    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.status, Status::Passed);

    // This proposal's deposit should not have been returned. It will
    // not be returnable until this is executed, or close on execution
    // is re-enabled.
    // Make sure the deposit was returned.
    let balance = query_balance_native(&app, CREATOR_ADDR, "ujuno");
    assert_eq!(balance, Uint128::new(0));
}

#[test]
fn test_reply_proposal_mock() {
    use crate::contract::reply;
    use crate::state::PROPOSALS;

    let mut deps = mock_dependencies();
    let env = mock_env();

    let m_proposal_id = mask_proposal_execution_proposal_id(1);
    PROPOSALS
        .save(
            deps.as_mut().storage,
            1,
            &SingleChoiceProposal {
                title: "A simple text proposal".to_string(),
                description: "This is a simple text proposal".to_string(),
                proposer: Addr::unchecked(CREATOR_ADDR),
                start_height: env.block.height,
                expiration: cw_utils::Duration::Height(6).after(&env.block),
                min_voting_period: None,
                threshold: Threshold::AbsolutePercentage {
                    percentage: PercentageThreshold::Majority {},
                },
                allow_revoting: false,
                total_power: Uint128::new(100_000_000),
                msgs: vec![],
                status: Status::Open,
                votes: Votes::zero(),
            },
        )
        .unwrap();

    // PROPOSALS
    let reply_msg = Reply {
        id: m_proposal_id,
        result: SubMsgResult::Err("error_msg".to_string()),
    };
    let res = reply(deps.as_mut(), env, reply_msg).unwrap();
    assert_eq!(
        res.attributes[0],
        Attribute {
            key: "proposal_execution_failed".to_string(),
            value: 1.to_string()
        }
    );

    let prop = PROPOSALS.load(deps.as_mut().storage, 1).unwrap();
    assert_eq!(prop.status, Status::ExecutionFailed);
}

#[test]
fn test_proposal_too_large() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let mut instantiate = get_proposal_module_instantiate(&mut app);
    instantiate.pre_propose_info = PreProposeInfo::AnyoneMayPropose {};
    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);
    let proposal_module = query_single_proposal_module(&app, &core_addr);

    let err = app
        .execute_contract(
            Addr::unchecked(CREATOR_ADDR),
            proposal_module,
            &ExecuteMsg::Propose {
                title: "".to_string(),
                description: "a".repeat(MAX_PROPOSAL_SIZE as usize),
                msgs: vec![],
                proposer: None,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert!(matches!(
        err,
        ContractError::ProposalTooLarge {
            size: _,
            max: MAX_PROPOSAL_SIZE
        }
    ))
}

#[test]
fn test_vote_not_registered() {
    let CommonTest {
        mut app,
        core_addr: _,
        proposal_module,
        proposal_id,
    } = setup_test(vec![]);

    let err =
        vote_on_proposal_should_fail(&mut app, &proposal_module, "ekez", proposal_id, Vote::Yes);
    assert!(matches!(err, ContractError::NotRegistered {}))
}

#[test]
fn test_proposal_creation_permissions() {
    let CommonTest {
        mut app,
        core_addr,
        proposal_module,
        proposal_id: _,
    } = setup_test(vec![]);

    // Non pre-propose may not propose.
    let err = app
        .execute_contract(
            Addr::unchecked("notprepropose"),
            proposal_module.clone(),
            &ExecuteMsg::Propose {
                title: "title".to_string(),
                description: "description".to_string(),
                msgs: vec![],
                proposer: None,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert!(matches!(err, ContractError::Unauthorized {}));

    let proposal_creation_policy = query_creation_policy(&app, &proposal_module);
    let pre_propose = match proposal_creation_policy {
        ProposalCreationPolicy::Anyone {} => panic!("expected a pre-propose module"),
        ProposalCreationPolicy::Module { addr } => addr,
    };

    // Proposer may not be none when a pre-propose module is making
    // the proposal.
    let err = app
        .execute_contract(
            pre_propose,
            proposal_module.clone(),
            &ExecuteMsg::Propose {
                title: "title".to_string(),
                description: "description".to_string(),
                msgs: vec![],
                proposer: None,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert!(matches!(err, ContractError::InvalidProposer {}));

    // Allow anyone to propose.
    app.execute_contract(
        core_addr,
        proposal_module.clone(),
        &ExecuteMsg::UpdatePreProposeInfo {
            info: PreProposeInfo::AnyoneMayPropose {},
        },
        &[],
    )
    .unwrap();

    // Proposer must be None when non pre-propose module is making the
    // proposal.
    let err = app
        .execute_contract(
            Addr::unchecked("ekez"),
            proposal_module.clone(),
            &ExecuteMsg::Propose {
                title: "title".to_string(),
                description: "description".to_string(),
                msgs: vec![],
                proposer: Some("ekez".to_string()),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert!(matches!(err, ContractError::InvalidProposer {}));

    // Works normally.
    let proposal_id = make_proposal(&mut app, &proposal_module, "ekez", vec![]);
    let proposal = query_proposal(&app, &proposal_module, proposal_id);
    assert_eq!(proposal.proposal.proposer, Addr::unchecked("ekez"));
    vote_on_proposal(
        &mut app,
        &proposal_module,
        CREATOR_ADDR,
        proposal_id,
        Vote::No,
    );
    close_proposal(&mut app, &proposal_module, CREATOR_ADDR, proposal_id);
}

#[test]
fn test_reply_hooks_mock() {
    use crate::contract::reply;
    use crate::state::{CREATION_POLICY, PROPOSAL_HOOKS, VOTE_HOOKS};

    let mut deps = mock_dependencies();
    let env = mock_env();

    // Add a proposal hook and remove it
    let m_proposal_hook_idx = mask_proposal_hook_index(0);
    PROPOSAL_HOOKS
        .add_hook(deps.as_mut().storage, Addr::unchecked(CREATOR_ADDR))
        .unwrap();

    let reply_msg = Reply {
        id: m_proposal_hook_idx,
        result: SubMsgResult::Err("error_msg".to_string()),
    };

    let res = reply(deps.as_mut(), env.clone(), reply_msg).unwrap();
    assert_eq!(
        res.attributes[0],
        Attribute {
            key: "removed_proposal_hook".to_string(),
            value: format! {"{CREATOR_ADDR}:{}", 0}
        }
    );

    // Reply needs a creation policy in state.
    CREATION_POLICY
        .save(
            deps.as_mut().storage,
            &ProposalCreationPolicy::Module {
                addr: Addr::unchecked("ekez"),
            },
        )
        .unwrap();

    let prepropose_reply_msg = Reply {
        id: failed_pre_propose_module_hook_id(),
        result: SubMsgResult::Err("error_msg".to_string()),
    };

    // Remove the pre-propose module as part of a reply.
    let res = reply(deps.as_mut(), env.clone(), prepropose_reply_msg.clone()).unwrap();
    assert_eq!(
        res.attributes[0],
        Attribute {
            key: "failed_prepropose_hook".to_string(),
            value: "ekez".into()
        }
    );

    // Do it again. This time, there is no module so this should error.
    let _id = failed_pre_propose_module_hook_id();
    let err = reply(deps.as_mut(), env.clone(), prepropose_reply_msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidReplyID { id: _ }));

    // Check that we fail open.
    let status = CREATION_POLICY.load(deps.as_ref().storage).unwrap();
    assert!(matches!(status, ProposalCreationPolicy::Anyone {}));

    // Vote hook
    let m_vote_hook_idx = mask_vote_hook_index(0);
    VOTE_HOOKS
        .add_hook(deps.as_mut().storage, Addr::unchecked(CREATOR_ADDR))
        .unwrap();

    let reply_msg = Reply {
        id: m_vote_hook_idx,
        result: SubMsgResult::Err("error_msg".to_string()),
    };
    let res = reply(deps.as_mut(), env, reply_msg).unwrap();
    assert_eq!(
        res.attributes[0],
        Attribute {
            key: "removed_vote_hook".to_string(),
            value: format! {"{CREATOR_ADDR}:{}", 0}
        }
    );
}

#[test]
fn test_query_info() {
    let CommonTest {
        app,
        core_addr: _,
        proposal_module,
        proposal_id: _,
    } = setup_test(vec![]);
    let info: InfoResponse = app
        .wrap()
        .query_wasm_smart(proposal_module, &QueryMsg::Info {})
        .unwrap();
    assert_eq!(
        info,
        InfoResponse {
            info: ContractVersion {
                contract: CONTRACT_NAME.to_string(),
                version: CONTRACT_VERSION.to_string()
            }
        }
    )
}

// Make a little multisig and test that queries to list votes work as
// expected.
#[test]
fn test_query_list_votes() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let mut instantiate = get_proposal_module_instantiate(&mut app);
    instantiate.threshold = Threshold::AbsoluteCount {
        threshold: Uint128::new(3),
    };
    instantiate.pre_propose_info = PreProposeInfo::AnyoneMayPropose {};
    let core_addr = instantiate_with_native_bonded_balances_governance(
        &mut app,
        instantiate,
        Some(vec![
            Cw20Coin {
                address: "one".to_string(),
                amount: Uint128::new(1),
            },
            Cw20Coin {
                address: "two".to_string(),
                amount: Uint128::new(1),
            },
            Cw20Coin {
                address: "three".to_string(),
                amount: Uint128::new(1),
            },
            Cw20Coin {
                address: "four".to_string(),
                amount: Uint128::new(1),
            },
            Cw20Coin {
                address: "five".to_string(),
                amount: Uint128::new(1),
            },
        ]),
    );
    let proposal_module = query_single_proposal_module(&app, &core_addr);
    let proposal_id = make_proposal(&mut app, &proposal_module, "one", vec![]);

    let votes = query_list_votes(&app, &proposal_module, proposal_id, None, None);
    assert_eq!(votes.votes, vec![]);

    vote_on_proposal(&mut app, &proposal_module, "two", proposal_id, Vote::No);
    vote_on_proposal(&mut app, &proposal_module, "three", proposal_id, Vote::No);
    vote_on_proposal(&mut app, &proposal_module, "one", proposal_id, Vote::Yes);
    vote_on_proposal(&mut app, &proposal_module, "four", proposal_id, Vote::Yes);
    vote_on_proposal(&mut app, &proposal_module, "five", proposal_id, Vote::Yes);

    let votes = query_list_votes(&app, &proposal_module, proposal_id, None, None);
    assert_eq!(
        votes.votes,
        vec![
            VoteInfo {
                voter: Addr::unchecked("five"),
                vote: Vote::Yes,
                power: Uint128::new(1)
            },
            VoteInfo {
                voter: Addr::unchecked("four"),
                vote: Vote::Yes,
                power: Uint128::new(1)
            },
            VoteInfo {
                voter: Addr::unchecked("one"),
                vote: Vote::Yes,
                power: Uint128::new(1)
            },
            VoteInfo {
                voter: Addr::unchecked("three"),
                vote: Vote::No,
                power: Uint128::new(1)
            },
            VoteInfo {
                voter: Addr::unchecked("two"),
                vote: Vote::No,
                power: Uint128::new(1)
            }
        ]
    );

    let votes = query_list_votes(
        &app,
        &proposal_module,
        proposal_id,
        Some("four".to_string()),
        Some(2),
    );
    assert_eq!(
        votes.votes,
        vec![
            VoteInfo {
                voter: Addr::unchecked("one"),
                vote: Vote::Yes,
                power: Uint128::new(1)
            },
            VoteInfo {
                voter: Addr::unchecked("three"),
                vote: Vote::No,
                power: Uint128::new(1)
            },
        ]
    );
}

/// DAO should be admin of the pre-propose contract despite the fact
/// that the proposal module instantiates it.
#[test]
fn test_pre_propose_admin_is_dao() {
    let CommonTest {
        app,
        core_addr,
        proposal_module,
        proposal_id: _,
    } = setup_test(vec![]);

    let proposal_creation_policy = query_creation_policy(&app, &proposal_module);

    // Check that a new creation policy has been birthed.
    let pre_propose = match proposal_creation_policy {
        ProposalCreationPolicy::Anyone {} => panic!("expected a pre-propose module"),
        ProposalCreationPolicy::Module { addr } => addr,
    };

    let info: ContractInfoResponse = app
        .wrap()
        .query(&cosmwasm_std::QueryRequest::Wasm(WasmQuery::ContractInfo {
            contract_addr: pre_propose.into_string(),
        }))
        .unwrap();
    assert_eq!(info.admin, Some(core_addr.into_string()));
}

#[test]
fn test_subdao_queries() {
    let mut app = custom_app::<NeutronMsg, Empty, _>(no_init);
    let instantiate = get_proposal_module_instantiate(&mut app);
    let core_addr = instantiate_with_native_bonded_balances_governance(&mut app, instantiate, None);

    let subdao_addr = Addr::unchecked("subdao");
    let res: StdResult<SubDao> = app.wrap().query_wasm_smart(
        core_addr.clone(),
        &DaoQueryMsg::GetSubDao {
            address: subdao_addr.to_string(),
        },
    );
    assert!(res.is_err());
    let res: Vec<SubDao> = app
        .wrap()
        .query_wasm_smart(
            core_addr.clone(),
            &DaoQueryMsg::ListSubDaos {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(res.len(), 0);

    let res = app.execute_contract(
        core_addr.clone(),
        core_addr.clone(),
        &DaoExecuteMsg::UpdateSubDaos {
            to_add: vec![SubDao {
                addr: subdao_addr.to_string(),
                charter: None,
            }],
            to_remove: vec![],
        },
        &[],
    );
    assert!(res.is_ok());
    let res: StdResult<SubDao> = app.wrap().query_wasm_smart(
        core_addr.clone(),
        &DaoQueryMsg::GetSubDao {
            address: subdao_addr.to_string(),
        },
    );
    assert!(res.is_ok());
    let res: Vec<SubDao> = app
        .wrap()
        .query_wasm_smart(
            core_addr,
            &DaoQueryMsg::ListSubDaos {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(res.len(), 1);
}

// TODO: test pre-propose module that fails on new proposal hook (ugh).

// - What happens if you have proposals that can not be executed but
//   took deposits and want to migrate?
