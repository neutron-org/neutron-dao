use crate::contract::query_proposal_execution_error;
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, Attribute, CosmosMsg, Reply, SubMsg, SubMsgResult, WasmMsg,
};
use cwd_voting::status::Status;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_subdao_core::msg::ExecuteMsg as CoreExecuteMsg;
use neutron_subdao_timelock_single::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    types::{Config, ProposalListResponse, ProposalStatus, SingleChoiceProposal},
};

use std::cell::RefCell;
use std::rc::Rc;

use crate::testing::mock_querier::{MOCK_MAIN_DAO_ADDR, MOCK_OVERRULE_PREPROPOSAL};
use crate::{
    contract::{execute, instantiate, query, reply},
    state::{CONFIG, DEFAULT_LIMIT, PROPOSALS},
    testing::mock_querier::MOCK_TIMELOCK_INITIALIZER,
};
use neutron_dao_pre_propose_overrule::msg::{
    ExecuteMsg as OverruleExecuteMsg, ProposeMessage as OverruleProposeMessage,
};

use super::mock_querier::{mock_dependencies, MOCK_SUBDAO_CORE_ADDR};

#[test]
fn test_instantiate_test() {
    let overrule_proposal_status = Rc::new(RefCell::new(Status::Open));
    let mut deps = mock_dependencies(Rc::clone(&overrule_proposal_status));
    let env = mock_env();
    let info = mock_info("neutron1unknownsender", &[]);
    let msg = InstantiateMsg {
        overrule_pre_propose: MOCK_OVERRULE_PREPROPOSAL.to_string(),
    };
    let res = instantiate(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(
        "Generic error: Querier system error: No such contract: neutron1unknownsender",
        res.unwrap_err().to_string()
    );

    let info = mock_info(MOCK_TIMELOCK_INITIALIZER, &[]);

    let msg = InstantiateMsg {
        overrule_pre_propose: MOCK_OVERRULE_PREPROPOSAL.to_string(),
    };
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    let res_ok = res.unwrap();
    let expected_attributes = vec![
        Attribute::new("action", "instantiate"),
        Attribute::new("owner", MOCK_MAIN_DAO_ADDR),
        Attribute::new("overrule_pre_propose", MOCK_OVERRULE_PREPROPOSAL),
    ];
    assert_eq!(expected_attributes, res_ok.attributes);
    let config = CONFIG.load(&deps.storage).unwrap();
    let expected_config = Config {
        owner: Addr::unchecked(MOCK_MAIN_DAO_ADDR),
        overrule_pre_propose: Addr::unchecked(msg.overrule_pre_propose),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    assert_eq!(expected_config, config);

    let msg = InstantiateMsg {
        overrule_pre_propose: MOCK_OVERRULE_PREPROPOSAL.to_string(),
    };
    let res = instantiate(deps.as_mut(), env, info, msg.clone());
    let res_ok = res.unwrap();
    let expected_attributes = vec![
        Attribute::new("action", "instantiate"),
        Attribute::new("owner", MOCK_MAIN_DAO_ADDR),
        Attribute::new("overrule_pre_propose", MOCK_OVERRULE_PREPROPOSAL),
    ];
    assert_eq!(expected_attributes, res_ok.attributes);
    let config = CONFIG.load(&deps.storage).unwrap();
    let expected_config = Config {
        owner: Addr::unchecked(MOCK_MAIN_DAO_ADDR),
        overrule_pre_propose: Addr::unchecked(msg.overrule_pre_propose),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    assert_eq!(expected_config, config);
}

#[test]
fn test_execute_timelock_proposal() {
    let overrule_proposal_status = Rc::new(RefCell::new(Status::Open));
    let mut deps = mock_dependencies(Rc::clone(&overrule_proposal_status));
    let env = mock_env();
    let info = mock_info("neutron1unknownsender", &[]);

    // No config set case
    let correct_msg = ExecuteMsg::TimelockProposal {
        proposal_id: 10,
        msgs: vec![correct_proposal_msg()],
    };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        correct_msg.clone(),
    );
    assert_eq!(
        "type: neutron_subdao_timelock_single::types::Config; key: [63, 6F, 6E, 66, 69, 67] not found",
        res.unwrap_err().to_string()
    );

    let config = Config {
        owner: Addr::unchecked("owner"),
        overrule_pre_propose: Addr::unchecked(MOCK_OVERRULE_PREPROPOSAL),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    // Unauthorized case
    let res = execute(deps.as_mut(), env.clone(), info, correct_msg.clone());
    assert_eq!("Unauthorized", res.unwrap_err().to_string());

    let info = mock_info(MOCK_SUBDAO_CORE_ADDR, &[]);

    // check that execution fails when there is a wrong type of message inside
    let incorrect_type_msg = ExecuteMsg::TimelockProposal {
        proposal_id: 10,
        msgs: vec![NeutronMsg::remove_interchain_query(1).into()],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), incorrect_type_msg);
    assert_eq!(
        "Can only execute msg of ExecuteTimelockedMsgs type",
        res.unwrap_err().to_string()
    );

    // check that execution fails when there are no messages inside
    let empty_msgs_msg = ExecuteMsg::TimelockProposal {
        proposal_id: 10,
        msgs: vec![],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), empty_msgs_msg);
    assert_eq!(
        "Can only execute proposals with exactly one message that of ExecuteTimelockedMsgs type. Got 0 messages.",
        res.unwrap_err().to_string()
    );

    // check that execution fails when there are 2 messages inside
    let too_many_msgs_msg = ExecuteMsg::TimelockProposal {
        proposal_id: 10,
        msgs: vec![correct_proposal_msg(), correct_proposal_msg()],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), too_many_msgs_msg);
    assert_eq!(
        "Can only execute proposals with exactly one message that of ExecuteTimelockedMsgs type. Got 2 messages.",
        res.unwrap_err().to_string()
    );

    // successful case
    let res_ok = execute(deps.as_mut(), env, info, correct_msg).unwrap();
    let expected_attributes = vec![
        Attribute::new("action", "timelock_proposal"),
        Attribute::new("sender", MOCK_SUBDAO_CORE_ADDR),
        Attribute::new("proposal_id", "10"),
        Attribute::new("status", "timelocked"),
    ];
    assert_eq!(expected_attributes, res_ok.attributes);
    assert_eq!(1, res_ok.messages.len());

    assert_eq!(
        res_ok.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_OVERRULE_PREPROPOSAL.to_string(),
            msg: to_json_binary(&OverruleExecuteMsg::Propose {
                msg: OverruleProposeMessage::ProposeOverrule {
                    timelock_contract: MOCK_CONTRACT_ADDR.to_string(),
                    proposal_id: 10,
                },
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    let expected_proposal = SingleChoiceProposal {
        id: 10,
        msgs: vec![correct_proposal_msg()],
        status: ProposalStatus::Timelocked,
    };
    let prop = PROPOSALS.load(deps.as_mut().storage, 10u64).unwrap();
    assert_eq!(expected_proposal, prop);
}

#[test]
fn test_execute_proposal() {
    let overrule_proposal_status = Rc::new(RefCell::new(Status::Open));
    let mut deps = mock_dependencies(Rc::clone(&overrule_proposal_status));
    let env = mock_env();
    let info = mock_info("neutron1unknownsender", &[]);

    let msg = ExecuteMsg::ExecuteProposal { proposal_id: 10 };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(
        "type: neutron_subdao_timelock_single::types::Config; key: [63, 6F, 6E, 66, 69, 67] not found",
        res.unwrap_err().to_string()
    );

    let config = Config {
        owner: Addr::unchecked("owner"),
        overrule_pre_propose: Addr::unchecked(MOCK_OVERRULE_PREPROPOSAL),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(
        "type: neutron_subdao_timelock_single::types::SingleChoiceProposal; key: [00, 09, 70, 72, 6F, 70, 6F, 73, 61, 6C, 73, 00, 00, 00, 00, 00, 00, 00, 0A] not found",
        res.unwrap_err().to_string()
    );

    let wrong_prop_statuses = vec![
        ProposalStatus::Executed,
        ProposalStatus::ExecutionFailed,
        ProposalStatus::Overruled,
    ];
    for s in wrong_prop_statuses {
        let proposal = SingleChoiceProposal {
            id: 10,
            msgs: vec![correct_proposal_msg()],
            status: s,
        };
        PROPOSALS
            .save(deps.as_mut().storage, proposal.id, &proposal)
            .unwrap();
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
        assert_eq!(
            format!("Wrong proposal status ({})", s),
            res.unwrap_err().to_string()
        )
    }

    // check execution with close_proposal_on_execution_failure = true
    deps.querier.set_close_proposal_on_execution_failure(true);
    let proposal = SingleChoiceProposal {
        id: 10,
        msgs: vec![correct_proposal_msg()],
        status: ProposalStatus::Timelocked,
    };
    PROPOSALS
        .save(deps.as_mut().storage, proposal.id, &proposal)
        .unwrap();
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!("Proposal is timelocked", res.unwrap_err().to_string());
    {
        let mut data_mut_ref = overrule_proposal_status.borrow_mut();
        *data_mut_ref = Status::Rejected;
    }
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    let expected_attributes = vec![
        Attribute::new("action", "execute_proposal"),
        Attribute::new("sender", "neutron1unknownsender"),
        Attribute::new("proposal_id", "10"),
    ];
    assert_eq!(expected_attributes, res.attributes);
    assert_eq!(
        proposal
            .msgs
            .iter()
            .map(|msg| SubMsg::reply_on_error(msg.clone(), proposal.id))
            .collect::<Vec<SubMsg<NeutronMsg>>>(),
        res.messages
    );
    let updated_prop = PROPOSALS.load(deps.as_mut().storage, 10).unwrap();
    assert_eq!(ProposalStatus::Executed, updated_prop.status);

    // check execution with close_proposal_on_execution_failure = true and overrule proposal Status == Closed
    deps.querier.set_close_proposal_on_execution_failure(true);
    let proposal = SingleChoiceProposal {
        id: 11,
        msgs: vec![correct_proposal_msg()],
        status: ProposalStatus::Timelocked,
    };
    PROPOSALS
        .save(deps.as_mut().storage, proposal.id, &proposal)
        .unwrap();
    // if overrule has Status::Closed that means it was rejected
    {
        let mut data_mut_ref = overrule_proposal_status.borrow_mut();
        *data_mut_ref = Status::Closed;
    }
    let msg2 = ExecuteMsg::ExecuteProposal { proposal_id: 11 };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg2.clone()).unwrap();
    let expected_attributes = vec![
        Attribute::new("action", "execute_proposal"),
        Attribute::new("sender", "neutron1unknownsender"),
        Attribute::new("proposal_id", "11"),
    ];
    assert_eq!(expected_attributes, res.attributes);
    assert_eq!(
        proposal
            .msgs
            .iter()
            .map(|msg| SubMsg::reply_on_error(msg.clone(), proposal.id))
            .collect::<Vec<SubMsg<NeutronMsg>>>(),
        res.messages
    );
    let updated_prop = PROPOSALS.load(deps.as_mut().storage, 11).unwrap();
    assert_eq!(ProposalStatus::Executed, updated_prop.status);

    // check that execution fails when there not exactly one message in proposal
    let proposal = SingleChoiceProposal {
        id: 10,
        msgs: vec![correct_proposal_msg(), correct_proposal_msg()],
        status: ProposalStatus::Timelocked,
    };
    PROPOSALS
        .save(deps.as_mut().storage, proposal.id, &proposal)
        .unwrap();
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(
        "Can only execute proposals with exactly one message that of ExecuteTimelockedMsgs type. Got 2 messages.",
        res.unwrap_err().to_string()
    );

    // check that execution fails when there is a wrong type of message inside
    let proposal = SingleChoiceProposal {
        id: 10,
        msgs: vec![NeutronMsg::remove_interchain_query(1).into()],
        status: ProposalStatus::Timelocked,
    };
    PROPOSALS
        .save(deps.as_mut().storage, proposal.id, &proposal)
        .unwrap();
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(
        "Can only execute msg of ExecuteTimelockedMsgs type",
        res.unwrap_err().to_string()
    );

    // check that execution fails when there are no messages inside
    let proposal = SingleChoiceProposal {
        id: 10,
        msgs: vec![],
        status: ProposalStatus::Timelocked,
    };
    PROPOSALS
        .save(deps.as_mut().storage, proposal.id, &proposal)
        .unwrap();
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(
        "Can only execute proposals with exactly one message that of ExecuteTimelockedMsgs type. Got 0 messages.",
        res.unwrap_err().to_string()
    );

    // check proposal execution close_proposal_on_execution_failure = false
    deps.querier.set_close_proposal_on_execution_failure(false);
    let proposal2 = SingleChoiceProposal {
        id: 10,
        msgs: vec![correct_proposal_msg()],
        status: ProposalStatus::Timelocked,
    };
    PROPOSALS
        .save(deps.as_mut().storage, proposal2.id, &proposal2)
        .unwrap();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    let expected_attributes = vec![
        Attribute::new("action", "execute_proposal"),
        Attribute::new("sender", "neutron1unknownsender"),
        Attribute::new("proposal_id", "10"),
    ];
    assert_eq!(expected_attributes, res.attributes);
    // added as messages without reply
    let expected_msgs = proposal2
        .msgs
        .iter()
        .map(|msg| SubMsg::new(msg.clone()))
        .collect::<Vec<SubMsg<NeutronMsg>>>();
    assert_eq!(expected_msgs, res.messages);
    let updated_prop_2 = PROPOSALS.load(deps.as_mut().storage, 10).unwrap();
    assert_eq!(ProposalStatus::Executed, updated_prop_2.status);
}

#[test]
fn test_overrule_proposal() {
    let overrule_proposal_status = Rc::new(RefCell::new(Status::Open));
    let mut deps = mock_dependencies(Rc::clone(&overrule_proposal_status));
    let env = mock_env();
    let info = mock_info("neutron1unknownsender", &[]);

    let msg = ExecuteMsg::OverruleProposal { proposal_id: 10 };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(
        "type: neutron_subdao_timelock_single::types::Config; key: [63, 6F, 6E, 66, 69, 67] not found",
        res.unwrap_err().to_string()
    );

    let config = Config {
        owner: Addr::unchecked("owner"),
        overrule_pre_propose: Addr::unchecked(MOCK_OVERRULE_PREPROPOSAL),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert_eq!("Unauthorized", res.unwrap_err().to_string());

    let info = mock_info("owner", &[]);

    let wrong_prop_statuses = vec![
        ProposalStatus::Executed,
        ProposalStatus::ExecutionFailed,
        ProposalStatus::Overruled,
    ];
    for s in wrong_prop_statuses {
        let proposal = SingleChoiceProposal {
            id: 10,
            msgs: vec![correct_proposal_msg()],
            status: s,
        };
        PROPOSALS
            .save(deps.as_mut().storage, proposal.id, &proposal)
            .unwrap();
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
        assert_eq!(
            format!("Wrong proposal status ({})", s),
            res.unwrap_err().to_string()
        )
    }

    let proposal = SingleChoiceProposal {
        id: 10,
        msgs: vec![correct_proposal_msg()],
        status: ProposalStatus::Timelocked,
    };
    PROPOSALS
        .save(deps.as_mut().storage, proposal.id, &proposal)
        .unwrap();
    let res_ok = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
    assert_eq!(0, res_ok.messages.len());
    let expected_attributes = vec![
        Attribute::new("action", "overrule_proposal"),
        Attribute::new("sender", info.sender),
        Attribute::new("proposal_id", proposal.id.to_string()),
    ];
    assert_eq!(expected_attributes, res_ok.attributes);
    let updated_prop = PROPOSALS.load(deps.as_mut().storage, 10).unwrap();
    assert_eq!(ProposalStatus::Overruled, updated_prop.status);
}

#[test]
fn execute_update_config() {
    let overrule_proposal_status = Rc::new(RefCell::new(Status::Open));
    let mut deps = mock_dependencies(Rc::clone(&overrule_proposal_status));
    let env = mock_env();
    let info = mock_info("neutron1unknownsender", &[]);

    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        overrule_pre_propose: Some("neutron1someotheroverrule".to_string()),
    };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(
        "type: neutron_subdao_timelock_single::types::Config; key: [63, 6F, 6E, 66, 69, 67] not found",
        res.unwrap_err().to_string()
    );

    let config = Config {
        owner: Addr::unchecked("owner"),
        overrule_pre_propose: Addr::unchecked(MOCK_OVERRULE_PREPROPOSAL),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert_eq!("Unauthorized", res.unwrap_err().to_string());

    let info = mock_info("owner", &[]);
    let config = Config {
        owner: Addr::unchecked("none"),
        overrule_pre_propose: Addr::unchecked(MOCK_OVERRULE_PREPROPOSAL),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!("Unauthorized", res.unwrap_err().to_string());

    let config = Config {
        owner: Addr::unchecked("owner"),
        overrule_pre_propose: Addr::unchecked(MOCK_OVERRULE_PREPROPOSAL),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let res_ok = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(0, res_ok.messages.len());
    let expected_attributes = vec![
        Attribute::new("action", "update_config"),
        Attribute::new("owner", "owner"),
        Attribute::new("overrule_pre_propose", "neutron1someotheroverrule"),
    ];
    assert_eq!(expected_attributes, res_ok.attributes);
    let updated_config = CONFIG.load(deps.as_mut().storage).unwrap();
    let some_other_prepropose = "neutron1someotheroverrule";
    assert_eq!(
        updated_config,
        Config {
            owner: Addr::unchecked("owner"),
            overrule_pre_propose: Addr::unchecked(some_other_prepropose),
            subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR)
        }
    );

    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("neutron1newowner".to_string()),
        overrule_pre_propose: None,
    };

    let res_ok = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(0, res_ok.messages.len());
    let expected_attributes = vec![
        Attribute::new("action", "update_config"),
        Attribute::new("owner", "neutron1newowner"),
        Attribute::new("overrule_pre_propose", some_other_prepropose),
    ];
    assert_eq!(expected_attributes, res_ok.attributes);
    let updated_config = CONFIG.load(deps.as_mut().storage).unwrap();
    assert_eq!(
        updated_config,
        Config {
            owner: Addr::unchecked("neutron1newowner"),
            overrule_pre_propose: Addr::unchecked(some_other_prepropose),
            subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR)
        }
    );

    // old owner
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!("Unauthorized", err.to_string());
}

#[test]
fn test_query_config() {
    let overrule_proposal_status = Rc::new(RefCell::new(Status::Open));
    let mut deps = mock_dependencies(Rc::clone(&overrule_proposal_status));
    let config = Config {
        owner: Addr::unchecked("owner"),
        overrule_pre_propose: Addr::unchecked(MOCK_OVERRULE_PREPROPOSAL),
        subdao: Addr::unchecked(MOCK_SUBDAO_CORE_ADDR),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let query_msg = QueryMsg::Config {};
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let queried_config: Config = from_json(res).unwrap();
    assert_eq!(config, queried_config)
}

#[test]
fn test_query_proposals() {
    let overrule_proposal_status = Rc::new(RefCell::new(Status::Open));
    let mut deps = mock_dependencies(Rc::clone(&overrule_proposal_status));
    for i in 1..=100 {
        let prop = SingleChoiceProposal {
            id: i,
            msgs: vec![correct_proposal_msg()],
            status: ProposalStatus::Timelocked,
        };
        PROPOSALS.save(deps.as_mut().storage, i, &prop).unwrap();
    }
    for i in 1..=100 {
        let query_msg = QueryMsg::Proposal { proposal_id: i };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let queried_prop: SingleChoiceProposal = from_json(&res).unwrap();
        let expected_prop = SingleChoiceProposal {
            id: i,
            msgs: vec![correct_proposal_msg()],
            status: ProposalStatus::Timelocked,
        };
        assert_eq!(expected_prop, queried_prop)
    }

    let query_msg = QueryMsg::ListProposals {
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let queried_props: ProposalListResponse = from_json(&res).unwrap();
    for (p, i) in queried_props.proposals.iter().zip(1..) {
        let expected_prop = SingleChoiceProposal {
            id: i,
            msgs: vec![correct_proposal_msg()],
            status: ProposalStatus::Timelocked,
        };
        assert_eq!(expected_prop, *p);
    }
    assert_eq!(queried_props.proposals.len(), DEFAULT_LIMIT as usize);

    let query_msg = QueryMsg::ListProposals {
        start_after: None,
        limit: Some(100),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let queried_props: ProposalListResponse = from_json(&res).unwrap();
    for (p, i) in queried_props.proposals.iter().zip(1..) {
        let expected_prop = SingleChoiceProposal {
            id: i,
            msgs: vec![correct_proposal_msg()],
            status: ProposalStatus::Timelocked,
        };
        assert_eq!(expected_prop, *p);
    }
    assert_eq!(queried_props.proposals.len(), 100);

    let query_msg = QueryMsg::ListProposals {
        start_after: None,
        limit: Some(10),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let queried_props: ProposalListResponse = from_json(&res).unwrap();
    for (p, i) in queried_props.proposals.iter().zip(1..) {
        let expected_prop = SingleChoiceProposal {
            id: i,
            msgs: vec![correct_proposal_msg()],
            status: ProposalStatus::Timelocked,
        };
        assert_eq!(expected_prop, *p);
    }
    assert_eq!(queried_props.proposals.len(), 10);

    let query_msg = QueryMsg::ListProposals {
        start_after: Some(50),
        limit: None,
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let queried_props: ProposalListResponse = from_json(&res).unwrap();
    for (p, i) in queried_props.proposals.iter().zip(51..) {
        let expected_prop = SingleChoiceProposal {
            id: i,
            msgs: vec![correct_proposal_msg()],
            status: ProposalStatus::Timelocked,
        };
        assert_eq!(expected_prop, *p);
    }
    assert_eq!(queried_props.proposals.len(), DEFAULT_LIMIT as usize);

    let query_msg = QueryMsg::ListProposals {
        start_after: Some(90),
        limit: None,
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let queried_props: ProposalListResponse = from_json(&res).unwrap();
    for (p, i) in queried_props.proposals.iter().zip(91..) {
        let expected_prop = SingleChoiceProposal {
            id: i,
            msgs: vec![correct_proposal_msg()],
            status: ProposalStatus::Timelocked,
        };
        assert_eq!(expected_prop, *p);
    }
    assert_eq!(queried_props.proposals.len(), 10);
}

#[test]
fn test_reply() {
    let overrule_proposal_status = Rc::new(RefCell::new(Status::Open));
    let mut deps = mock_dependencies(Rc::clone(&overrule_proposal_status));
    let msg = Reply {
        id: 10,
        result: SubMsgResult::Err("error".to_string()),
    };
    let err = reply(deps.as_mut(), mock_env(), msg.clone()).unwrap_err();
    assert_eq!("No such proposal (10)", err.to_string());

    let prop = SingleChoiceProposal {
        id: 10,
        msgs: vec![correct_proposal_msg()],
        status: ProposalStatus::Timelocked,
    };
    let env = mock_env();
    PROPOSALS.save(deps.as_mut().storage, 10, &prop).unwrap();
    let res_ok = reply(deps.as_mut(), env, msg).unwrap();
    assert_eq!(0, res_ok.messages.len());
    let expected_attributes = vec![Attribute::new("timelocked_proposal_execution_failed", "10")];
    assert_eq!(expected_attributes, res_ok.attributes);
    // reply writes the failed proposal error
    let query_res = query_proposal_execution_error(deps.as_ref(), 10).unwrap();
    let error: Option<String> = from_json(query_res).unwrap();
    assert_eq!(error, Some("error".to_string()));
}

fn correct_proposal_msg() -> CosmosMsg<NeutronMsg> {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: "".to_string(),
        msg: to_json_binary(&CoreExecuteMsg::ExecuteTimelockedMsgs { msgs: vec![] }).unwrap(),
        funds: vec![],
    })
}
