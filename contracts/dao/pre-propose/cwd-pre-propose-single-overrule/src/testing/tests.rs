use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    to_binary, CosmosMsg, DepsMut, Empty, SubMsg, WasmMsg,
};
use cwd_core::query::SubDao;
use std::collections::HashMap;

use crate::{
    contract::{execute, instantiate, query},
    testing::mock_querier::{
        mock_dependencies, MOCK_DAO_CORE, MOCK_SUBDAO_PROPOSE_MODULE, MOCK_TIMELOCK_CONTRACT,
    },
};
use neutron_dao_pre_propose_overrule::msg::{
    ExecuteMsg, InstantiateMsg, ProposeMessageInternal, QueryMsg,
};

use crate::error::PreProposeOverruleError;
use crate::testing::mock_querier::{
    get_dao_with_impostor_subdao, get_dao_with_impostor_timelock, get_dao_with_many_subdaos,
    get_properly_initialized_dao, ContractQuerier, MockDaoProposalQueries, MockDaoQueries,
    MockSubaoPreProposalQueries, MockSubdaoCoreQueries, MockSubdaoProposalQueries,
    MockTimelockQueries, MOCK_DAO_PROPOSE_MODULE, MOCK_IMPOSTOR_TIMELOCK_CONTRACT,
    MOCK_SUBDAO_CORE, MOCK_SUBDAO_PREPROPOSE_MODULE, MOCK_TIMELOCK_CONTRACT_IMPOSTOR_SUBDAO,
    NON_TIMELOCKED_PROPOSAL_ID, SUBDAO_NAME, TIMELOCKED_PROPOSAL_ID,
};
use cwd_pre_propose_base::state::Config;
use neutron_dao_pre_propose_overrule::types::ProposeMessage;
use neutron_subdao_timelock_single::msg as TimelockMsg;

pub fn init_base_contract(deps: DepsMut<Empty>) {
    let msg = InstantiateMsg {
        main_dao: MOCK_DAO_CORE.to_string(),
    };
    let info = mock_info(MOCK_DAO_PROPOSE_MODULE, &[]);
    instantiate(deps, mock_env(), info, msg).unwrap();
}

#[test]
fn test_create_overrule_proposal() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    println!("{:?}", res);
    assert!(res.is_ok());
    let prop_desc: String = format!("Reject the decision made by the {} subdao", SUBDAO_NAME);
    let prop_name: String = format!("Overrule proposal {} of {}", PROPOSAL_ID, SUBDAO_NAME);
    assert_eq!(
        res.unwrap().messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_DAO_PROPOSE_MODULE.to_string(),
            msg: to_binary(&ProposeMessageInternal::Propose {
                title: prop_name,
                description: prop_desc,
                msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_TIMELOCK_CONTRACT.to_string(),
                    msg: to_binary(&TimelockMsg::ExecuteMsg::OverruleProposal {
                        proposal_id: PROPOSAL_ID
                    })
                    .unwrap(),
                    funds: vec![],
                })],
                proposer: Some(PROPOSER_ADDR.to_string()),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );
}

#[test]
fn test_query_config() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    let query_msg = QueryMsg::Config {};
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let queried_prop = from_binary(&res).unwrap();
    let expected_prop = Config {
        deposit_info: None,
        open_proposal_submission: true,
    };
    assert_eq!(expected_prop, queried_prop);
}

#[test]
fn test_base_prepropose_methods() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::UpdateConfig {
        deposit_info: None,
        open_proposal_submission: true,
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap(),
        PreProposeOverruleError::MessageUnsupported {}
    )
}

#[test]
fn test_impostor_subdao() {
    // test where timelock contract points to subdao that doesn't points to this timelock
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_dao_with_impostor_subdao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res_not_ok = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res_not_ok.is_err());
    assert_eq!(res_not_ok, Err(PreProposeOverruleError::ForbiddenSubdao {}));
}

#[test]
fn test_impostor_timelock() {
    // test where timelock contract points to subdao that doesn't points to this timelock
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_dao_with_impostor_timelock();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_IMPOSTOR_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res_not_ok = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res_not_ok.is_err());
    assert_eq!(
        res_not_ok,
        Err(PreProposeOverruleError::SubdaoMisconfured {})
    );
}

#[test]
fn test_proposal_is_not_timelocked() {
    // test where the proposal we're to create overrule for isn't timelocked already/yet
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = HashMap::new();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = NON_TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res_not_ok = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res_not_ok.is_err());
    assert_eq!(
        res_not_ok,
        Err(PreProposeOverruleError::ProposalWrongState {})
    );
}

#[test]
fn test_long_subdao_list() {
    // test where we check if out pagination handling works properly
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_dao_with_many_subdaos();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    println!("{:?}", res);
    assert!(res.is_ok());
}

#[test]
fn test_double_creation() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res_ok = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg.clone(),
    );
    assert!(res_ok.is_ok());
    let res_not_ok = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res_not_ok.is_err());
    match res_not_ok {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            assert_eq!(err, PreProposeOverruleError::AlreadyExists {})
        }
    }
}
