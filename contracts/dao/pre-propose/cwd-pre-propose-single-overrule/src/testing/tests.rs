use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    to_binary, CosmosMsg, DepsMut, Empty, SubMsg, WasmMsg,
};

use crate::{
    contract::{
        execute, instantiate, query, ExecuteMsg, InstantiateMsg, ProposeMessage,
        ProposeMessageInternal, QueryMsg, TimelockExecuteMsg,
    },
    testing::mock_querier::{mock_dependencies, MOCK_PROPOSE_MODULE, MOCK_TIMELOCK_CONTRACT},
};

use crate::error::PreProposeOverruleError;
use cwd_pre_propose_base::state::Config;

pub fn init_base_contract(deps: DepsMut<Empty>) {
    let msg = InstantiateMsg {};
    let info = mock_info(MOCK_PROPOSE_MODULE, &[]);
    instantiate(deps, mock_env(), info, msg).unwrap();
}

#[test]
fn test_create_overrule_proposal() {
    let mut deps = mock_dependencies();
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = 47;
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
    assert!(res.is_ok());
    assert_eq!(
        res.unwrap().messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_PROPOSE_MODULE.to_string(),
            msg: to_binary(&ProposeMessageInternal::Propose {
                title: "Overrule proposal".to_string(),
                description: "Reject the decision made by subdao".to_string(),
                msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_TIMELOCK_CONTRACT.to_string(),
                    msg: to_binary(&TimelockExecuteMsg::OverruleProposal {
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
    let mut deps = mock_dependencies();
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
    let mut deps = mock_dependencies();
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
