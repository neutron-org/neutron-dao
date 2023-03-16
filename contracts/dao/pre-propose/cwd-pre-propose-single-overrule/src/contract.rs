#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};
use cw2::set_contract_version;
use error::PreProposeOverruleError;

use crate::error;
use neutron_dao_pre_propose_overrule::msg::{
    ExecuteMsg, InstantiateMsg, ProposeMessageInternal, QueryMsg,
};
// use crate::state::{Config, CONFIG};
use cwd_pre_propose_base::{
    error::PreProposeError,
    msg::{ExecuteMsg as ExecuteBase, InstantiateMsg as InstantiateBase},
    state::PreProposeContract,
};

use crate::state::PROPOSALS;
use cwd_core::{msg::QueryMsg as MainDaoQueryMsg, query::SubDao};
use cwd_proposal_single::msg::QueryMsg as ProposalSingleQueryMsg;
use cwd_voting::pre_propose::ProposalCreationPolicy;
use neutron_dao_pre_propose_overrule::types::ProposeMessage;
use neutron_subdao_core::{msg::QueryMsg as SubdaoQueryMsg, types as SubdaoTypes};
use neutron_subdao_pre_propose_single::msg::QueryMsg as SubdaoPreProposeQueryMsg;
use neutron_subdao_proposal_single::msg as SubdaoProposalMsg;
use neutron_subdao_timelock_single::{msg as TimelockMsg, types as TimelockTypes};

pub(crate) const CONTRACT_NAME: &str = "crates.io:cwd-pre-propose-single-overrule";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type PrePropose = PreProposeContract<ProposeMessageInternal>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, PreProposeError> {
    // the contract has no info for instantiation so far, so it just calls the init function of base
    // deposit is set to zero because it makes no sense for overrule proposals
    // for open submission it's tbd
    let resp = PrePropose::default().instantiate(
        deps.branch(),
        env,
        info,
        InstantiateBase {
            deposit_info: None,
            open_proposal_submission: true,
        },
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, PreProposeOverruleError> {
    // We don't want to expose the `proposer` field on the propose
    // message externally as that is to be set by this module. Here,
    // we transform an external message which omits that field into an
    // internal message which sets it.
    type ExecuteInternal = ExecuteBase<ProposeMessageInternal>;
    match msg {
        ExecuteMsg::Propose {
            msg:
                ProposeMessage::ProposeOverrule {
                    timelock_contract,
                    proposal_id,
                },
        } => {
            let timelock_contract_addr = deps.api.addr_validate(&timelock_contract)?;

            if PROPOSALS
                .load(deps.storage, (proposal_id, timelock_contract_addr.clone()))
                .is_ok()
            {
                return Err(PreProposeOverruleError::AlreadyExists {});
            }

            let subdao_address = get_subdao_from_timelock(&deps, &timelock_contract)?;

            // we need this check since the timelock contract might be an impostor
            if get_timelock_from_subdao(&deps, &subdao_address)? != timelock_contract {
                return Err(PreProposeOverruleError::SubdaoMisconfured {});
            }

            if !check_if_subdao_legit(&deps, &subdao_address)? {
                return Err(PreProposeOverruleError::ForbiddenSubdao {});
            }

            if !check_is_proposal_timelocked(
                &deps,
                &Addr::unchecked(timelock_contract_addr.clone()),
                proposal_id,
            )? {
                return Err(PreProposeOverruleError::ProposalWrongState {});
            }

            let overrule_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: timelock_contract_addr.to_string(),
                msg: to_binary(&TimelockMsg::ExecuteMsg::OverruleProposal { proposal_id })?,
                funds: vec![],
            });

            let subdao_name = get_subdao_name(&deps, &subdao_address)?;
            let prop_desc: String =
                format!("Reject the decision made by the {} subdao", subdao_name);
            let prop_name: String = format!("Overrule proposal {} of {}", proposal_id, subdao_name);

            let internal_msg = ExecuteInternal::Propose {
                msg: ProposeMessageInternal::Propose {
                    // Fill in proposer based on message sender.
                    proposer: Some(info.sender.to_string()),
                    title: prop_name,
                    description: prop_desc,
                    msgs: vec![overrule_msg],
                },
            };

            let next_proposal_id = &get_next_proposal_id(&deps)?;

            PROPOSALS.save(
                deps.storage,
                (proposal_id, timelock_contract_addr.clone()),
                next_proposal_id,
            )?;

            PrePropose::default()
                .execute(deps, env, info, internal_msg)
                .map_err(|e| PreProposeOverruleError::PreProposeBase(e))
        }
        _ => Err(PreProposeOverruleError::MessageUnsupported {}),
    }
}

fn get_subdao_from_timelock(
    deps: &DepsMut,
    timelock_contract: &String,
) -> Result<Addr, PreProposeOverruleError> {
    let timelock_config: TimelockTypes::Config = deps.querier.query_wasm_smart(
        timelock_contract.to_string(),
        &TimelockMsg::QueryMsg::Config {},
    )?;
    Ok(timelock_config.subdao)
}

fn get_timelock_from_subdao(
    deps: &DepsMut,
    subdao_core: &Addr,
) -> Result<Addr, PreProposeOverruleError> {
    let proposal_modules: Vec<SubdaoTypes::ProposalModule> = deps.querier.query_wasm_smart(
        subdao_core,
        &SubdaoQueryMsg::ProposalModules {
            start_after: None,
            // we assume any subdao proposal module has pre-propose module with timelock.
            // thus, we need only single module
            limit: Some(1),
        },
    )?;

    if proposal_modules.is_empty() {
        return Err(PreProposeOverruleError::SubdaoMisconfured {});
    }

    let prop_policy: ProposalCreationPolicy = deps.querier.query_wasm_smart(
        proposal_modules.first().unwrap().address.clone(),
        &SubdaoProposalMsg::QueryMsg::ProposalCreationPolicy {},
    )?;

    match prop_policy {
        ProposalCreationPolicy::Anyone {} => Err(PreProposeOverruleError::SubdaoMisconfured {}),
        ProposalCreationPolicy::Module { addr } => {
            let timelock: Addr = deps
                .querier
                .query_wasm_smart(addr, &SubdaoPreProposeQueryMsg::TimelockAddress {})?;
            Ok(timelock)
        }
    }
}

fn check_if_subdao_legit(
    deps: &DepsMut,
    subdao_core: &Addr,
) -> Result<bool, PreProposeOverruleError> {
    let main_dao = get_main_dao_address(&deps)?;

    let mut start_after: Option<&SubDao> = None;
    let query_limit = 10;

    // unfortunately, there is no way to get the total subdao number so we do infinite loop here
    loop {
        let subdao_list: Vec<SubDao> = deps.querier.query_wasm_smart(
            main_dao.clone(),
            &MainDaoQueryMsg::ListSubDaos {
                start_after: match start_after.clone() {
                    None => None,
                    Some(a) => Some(a.clone().addr),
                },
                limit: Some(query_limit),
            },
        )?;

        if subdao_list.is_empty() {
            return Ok(false);
        }

        start_after = subdao_list.last();

        if subdao_list
            .into_iter()
            .find(|subdao| subdao.addr == subdao_core.clone().into_string())
            .is_some()
        {
            return Ok(true);
        };

        if subdao_list.len() < query_limit as usize {
            return Ok(false);
        }
    }
}

fn get_main_dao_address(deps: &DepsMut) -> Result<Addr, PreProposeOverruleError> {
    let dao: Addr = deps.querier.query_wasm_smart(
        PrePropose::default().proposal_module.load(deps.storage)?,
        &ProposalSingleQueryMsg::Dao {},
    )?;
    Ok(dao)
}

fn get_next_proposal_id(deps: &DepsMut) -> Result<u64, PreProposeOverruleError> {
    let last_proposal_id: u64 = deps.querier.query_wasm_smart(
        PrePropose::default().proposal_module.load(deps.storage)?,
        &ProposalSingleQueryMsg::ProposalCount {},
    )?;
    Ok(last_proposal_id + 1)
}

fn check_is_proposal_timelocked(
    deps: &DepsMut,
    timelock: &Addr,
    proposal_id: u64,
) -> Result<bool, PreProposeOverruleError> {
    let proposal: TimelockTypes::SingleChoiceProposal = deps
        .querier
        .query_wasm_smart(timelock, &TimelockMsg::QueryMsg::Proposal { proposal_id })?;
    return Ok(proposal.status == TimelockTypes::ProposalStatus::Timelocked);
}

fn get_subdao_name(deps: &DepsMut, subdao: &Addr) -> Result<String, PreProposeOverruleError> {
    let subdao_config: SubdaoTypes::Config = deps
        .querier
        .query_wasm_smart(subdao, &SubdaoQueryMsg::Config {})?;
    Ok(subdao_config.name)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    PrePropose::default().query(deps, env, msg)
}
