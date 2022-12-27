use crate::{
    contract::{execute, query},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    testing::mock_querier::mock_dependencies,
};
use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    to_binary, Addr, Uint128,
};
use cw4_voting::msg::InstantiateMsg as VoteModuleInstantiateMsg;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use cwd_interface::ModuleInstantiateInfo;
use cwd_subdao_pre_propose_single::contract::InstantiateMsg as PreProposeSingleInstantiateMsg;
use cwd_subdao_proposal_single::msg::InstantiateMsg as ProposalSingleInstantiateMsg;
use cwd_subdao_timelock_single::msg::InstantiateMsg as TimelockSingleInstantiateMsg;
use cwd_voting::{pre_propose::PreProposeInfo::ModuleMayPropose, threshold::Threshold};
use exec_control::pause::{PauseError, PauseInfoResponse};
use neutron_bindings::bindings::msg::NeutronMsg;

const NAME: &str = "subdao_name";
const DESCRIPTION: &str = "";
const SUBDAO_URI: &str = "http://testsubdao.neutron.org";
const DENOM: &str = "denom";
const MAIN_DAO_ADDR: &str = "main_dao";
const SECURITY_DAO_ADDR: &str = "security_dao";

const MEMBER_1_ADDR: &str = "member_1";
const MEMBER_2_ADDR: &str = "member_2";

fn cwd_subdao_core_contract() -> Box<dyn Contract<NeutronMsg>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply)
    .with_migrate(crate::contract::migrate);
    Box::new(contract)
}

pub fn init_base_contract(app: &mut App) -> Addr {
    let vote_module_instantiate = VoteModuleInstantiateMsg {
        cw4_group_code_id: 42,
        initial_members: vec![
            cw4::Member {
                addr: MEMBER_1_ADDR.to_string(),
                weight: 1,
            },
            cw4::Member {
                addr: MEMBER_2_ADDR.to_string(),
                weight: 1,
            },
        ],
    };

    let timelock_instantiate = TimelockSingleInstantiateMsg {
        owner: Some(Addr::unchecked(MAIN_DAO_ADDR)),
        timelock_duration: 1000,
    };

    let pre_propose_instantiate = PreProposeSingleInstantiateMsg {
        deposit_info: None,
        open_proposal_submission: true,
        timelock_module_instantiate_info: ModuleInstantiateInfo {
            code_id: 42,
            msg: to_binary(&timelock_instantiate).unwrap(),
            admin: None,
            label: String::from("timelock"),
        },
    };

    let proposal_module_instantiate = ProposalSingleInstantiateMsg {
        threshold: Threshold::AbsoluteCount {
            threshold: Uint128::from(66u32),
        },
        max_voting_period: cw_utils::Duration::Height(100),
        min_voting_period: None,
        allow_revoting: false,
        pre_propose_info: ModuleMayPropose {
            info: ModuleInstantiateInfo {
                code_id: 42,
                msg: to_binary(&pre_propose_instantiate).unwrap(),
                admin: None,
                label: String::from("prepropose"),
            },
        },
        close_proposal_on_execution_failure: true,
    };

    let msg = InstantiateMsg {
        name: NAME.to_string(),
        description: DESCRIPTION.to_string(),
        vote_module_instantiate_info: ModuleInstantiateInfo {
            code_id: 42,
            msg: to_binary(&vote_module_instantiate).unwrap(),
            admin: None,
            label: String::from("vote"),
        },
        proposal_modules_instantiate_info: vec![ModuleInstantiateInfo {
            code_id: 42,
            msg: to_binary(&proposal_module_instantiate).unwrap(),
            admin: None,
            label: String::from("proposal"),
        }],
        initial_items: None,
        dao_uri: Some(SUBDAO_URI.to_string()),
        security_dao: Addr::unchecked(SECURITY_DAO_ADDR),
    };

    let subdao_id = app.store_code(cwd_subdao_core_contract());
    app.instantiate_contract(
        subdao_id,
        Addr::unchecked(MAIN_DAO_ADDR),
        &msg,
        &[],
        "subdao",
        None,
    )
    .unwrap()
}

#[test]
fn test_pause() {
    let mut deps = mock_dependencies(&[]);
    let mut app = App::default();
    let subdao_addr = init_base_contract(&mut app);
    println!("SUBDAO addr: {}", subdao_addr);

    // pause contracts for 10 blocks from main dao
    let msg = ExecuteMsg::Pause { duration: 10u64 };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());
    let pause_info: PauseInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::PauseInfo {}).unwrap()).unwrap();
    assert_eq!(
        pause_info,
        PauseInfoResponse::Paused {
            until_height: mock_env().block.height + 10
        }
    );

    // security dao can't unpause contracts
    let msg = ExecuteMsg::Unpause {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(SECURITY_DAO_ADDR, &[]),
        msg,
    );
    assert_eq!(
        res.err().unwrap(),
        ContractError::PauseError(PauseError::Unauthorized {})
    );

    // unable to execute anything
    let msg = ExecuteMsg::SetItem {
        key: String::from("key"),
        addr: String::from("value"),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert_eq!(
        res.err().unwrap(),
        ContractError::PauseError(PauseError::Paused {})
    );

    let mut env = mock_env();
    env.block.height += 11;

    // but we can do it after 11 blocks
    let msg = ExecuteMsg::SetItem {
        key: String::from("key"),
        addr: String::from("value"),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    )
    .unwrap();

    env.block.height += 15;

    // pause contracts for 10 blocks from security dao
    let msg = ExecuteMsg::Pause { duration: 10u64 };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(SECURITY_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());
    let pause_info: PauseInfoResponse =
        from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::PauseInfo {}).unwrap()).unwrap();
    assert_eq!(
        pause_info,
        PauseInfoResponse::Paused {
            until_height: env.block.height + 10
        }
    );

    // only main dao can unpause contracts
    let msg = ExecuteMsg::Unpause {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MAIN_DAO_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok(),);
    let pause_info: PauseInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::PauseInfo {}).unwrap()).unwrap();
    assert_eq!(pause_info, PauseInfoResponse::Unpaused {});
}
