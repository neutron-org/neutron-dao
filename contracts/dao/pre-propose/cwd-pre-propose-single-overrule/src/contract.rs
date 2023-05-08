#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};
use cw2::set_contract_version;
use error::PreProposeOverruleError;

use crate::error;
use cwd_pre_propose_base::{
    error::PreProposeError,
    msg::{ExecuteMsg as ExecuteBase, InstantiateMsg as InstantiateBase},
    state::PreProposeContract,
};
use neutron_dao_pre_propose_overrule::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, ProposeMessage, QueryExt, QueryMsg,
};

use crate::state::PROPOSALS;
use cwd_core::{msg::QueryMsg as MainDaoQueryMsg, query::SubDao};
use cwd_proposal_single::{
    msg::ExecuteMsg as ProposeMessageInternal, msg::QueryMsg as ProposalSingleQueryMsg,
};
use cwd_voting::pre_propose::ProposalCreationPolicy;
use neutron_subdao_core::{msg::QueryMsg as SubdaoQueryMsg, types as SubdaoTypes};
use neutron_subdao_pre_propose_single::msg::{
    QueryExt as SubdaoPreProposeQueryExt, QueryMsg as SubdaoPreProposeQueryMsg,
};
use neutron_subdao_proposal_single::msg as SubdaoProposalMsg;
use neutron_subdao_timelock_single::{msg as TimelockMsg, types as TimelockTypes};

pub(crate) const CONTRACT_NAME: &str = "crates.io:cwd-pre-propose-single-overrule";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type PrePropose = PreProposeContract<ProposeMessageInternal, QueryExt>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, PreProposeError> {
    // The contract has no info for instantiation so far, so it just calls the init function of base
    let resp = PrePropose::default().instantiate(
        deps.branch(),
        env,
        info,
        InstantiateBase {
            // We restrict deposits since overrule proposals are supposed to be created automatically
            deposit_info: None,
            // Actually, the overrule proposal is going to be created by the timelock contract which
            // is not the DAO member and has no voting power.
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
    type ExecuteInternal = ExecuteBase<ProposeMessageInternal>;
    let internal_msg = match msg {
        ExecuteMsg::Propose {
            msg:
                ProposeMessage::ProposeOverrule {
                    timelock_contract,
                    proposal_id,
                },
        } => {
            let timelock_contract_addr = deps.api.addr_validate(&timelock_contract)?;

            let subdao_address = get_subdao_from_timelock(&deps, &timelock_contract_addr)?;
            perform_checks(&deps, &subdao_address, &timelock_contract_addr, proposal_id)?;

            let overrule_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: timelock_contract_addr.to_string(),
                msg: to_binary(&TimelockMsg::ExecuteMsg::OverruleProposal { proposal_id })?,
                funds: vec![],
            });

            let subdao_name = get_subdao_name(&deps, &subdao_address)?;
            let prop_name: String = format!(
                "Reject the proposal #{} of the '{}' subdao",
                proposal_id, subdao_name
            );
            let prop_desc: String = format!(
                "If this proposal will be accepted, the DAO is going to \
overrule the proposal #{} of '{}' subdao (address {})",
                proposal_id, subdao_name, subdao_address
            );

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
                (proposal_id, timelock_contract_addr),
                next_proposal_id,
            )?;

            Ok(internal_msg)
        }
        // The following messages are forwarded to the base contract
        ExecuteMsg::ProposalCreatedHook {
            proposal_id,
            proposer,
        } => Ok(ExecuteInternal::ProposalCreatedHook {
            proposal_id,
            proposer,
        }),
        ExecuteMsg::ProposalCompletedHook {
            proposal_id,
            new_status,
        } => Ok(ExecuteInternal::ProposalCompletedHook {
            proposal_id,
            new_status,
        }),
        // ExecuteMsg::Withdraw and ExecuteMsg::UpdateConfig are unsupported
        // ExecuteMsg::Withdraw is unsupported because overrule proposals should have no deposits
        // ExecuteMsg::UpdateConfig since the config has only the info about deposits,
        // no custom fields are added.
        _ => Err(PreProposeOverruleError::MessageUnsupported {}),
    };
    PrePropose::default()
        .execute(deps, env, info, internal_msg?)
        .map_err(PreProposeOverruleError::PreProposeBase)
}

fn get_subdao_from_timelock(
    deps: &DepsMut,
    timelock_contract: &Addr,
) -> Result<Addr, PreProposeOverruleError> {
    let timelock_config: TimelockTypes::Config = deps.querier.query_wasm_smart(
        timelock_contract.to_string(),
        &TimelockMsg::QueryMsg::Config {},
    )?;
    Ok(timelock_config.subdao)
}

fn verify_is_timelock_from_subdao(
    deps: &DepsMut,
    subdao_core: &Addr,
    expected_timelock: &Addr,
) -> Result<(), PreProposeOverruleError> {
    let proposal_modules: Vec<SubdaoTypes::ProposalModule> = deps.querier.query_wasm_smart(
        subdao_core,
        // we do no pagination here since it either fits in tx by gas or not
        &SubdaoQueryMsg::ProposalModules {
            start_after: None,
            limit: None,
        },
    )?;

    for proposal_module in proposal_modules {
        let prop_policy: ProposalCreationPolicy = deps.querier.query_wasm_smart(
            proposal_module.address,
            &SubdaoProposalMsg::QueryMsg::ProposalCreationPolicy {},
        )?;
        if let ProposalCreationPolicy::Module { addr } = prop_policy {
            if let Ok(timelock) = deps.querier.query_wasm_smart::<Addr>(
                addr,
                &SubdaoPreProposeQueryMsg::QueryExtension {
                    msg: SubdaoPreProposeQueryExt::TimelockAddress {},
                },
            ) {
                if *expected_timelock == timelock {
                    return Ok(());
                }
            }
        }
    }

    Err(PreProposeOverruleError::SubdaoMisconfigured {})
}

fn is_subdao_legit(deps: &DepsMut, subdao_core: &Addr) -> Result<(), PreProposeOverruleError> {
    let main_dao = get_main_dao_address(deps)?;

    let subdao: StdResult<SubDao> = deps.querier.query_wasm_smart(
        main_dao,
        &MainDaoQueryMsg::GetSubDao {
            address: subdao_core.to_string(),
        },
    );

    match subdao {
        Ok(subdao) => {
            if subdao.addr == *subdao_core {
                Ok(())
            } else {
                Err(PreProposeOverruleError::SubdaoMisconfigured {})
            }
        }
        Err(_) => Err(PreProposeOverruleError::ForbiddenSubdao {}),
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

fn is_proposal_timelocked(
    deps: &DepsMut,
    timelock: &Addr,
    proposal_id: u64,
) -> Result<(), PreProposeOverruleError> {
    let proposal: TimelockTypes::SingleChoiceProposal = deps
        .querier
        .query_wasm_smart(timelock, &TimelockMsg::QueryMsg::Proposal { proposal_id })?;
    if proposal.status == TimelockTypes::ProposalStatus::Timelocked {
        Ok(())
    } else {
        Err(PreProposeOverruleError::ProposalWrongState {})
    }
}

fn perform_checks(
    deps: &DepsMut,
    subdao: &Addr,
    timelock: &Addr,
    proposal_id: u64,
) -> Result<(), PreProposeOverruleError> {
    if let Ok(id) = PROPOSALS.load(deps.storage, (proposal_id, timelock.clone())) {
        return Err(PreProposeOverruleError::AlreadyExists { id });
    }

    // We need this check since the timelock contract might be an impostor
    // E.g. the timelock contract might be a malicious contract that is not a part of
    // the subdao but pretends to be.
    verify_is_timelock_from_subdao(deps, subdao, timelock)?;

    is_subdao_legit(deps, subdao)?;

    is_proposal_timelocked(deps, timelock, proposal_id)?;

    Ok(())
}

fn get_subdao_name(deps: &DepsMut, subdao: &Addr) -> Result<String, PreProposeOverruleError> {
    let subdao_config: SubdaoTypes::Config = deps
        .querier
        .query_wasm_smart(subdao, &SubdaoQueryMsg::Config {})?;
    Ok(subdao_config.name)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryExtension {
            msg:
                QueryExt::OverruleProposalId {
                    timelock_address,
                    subdao_proposal_id,
                },
        } => {
            let overrule_proposal_id = PROPOSALS.load(
                deps.storage,
                (
                    subdao_proposal_id,
                    deps.api.addr_validate(&timelock_address)?,
                ),
            )?;
            to_binary(&overrule_proposal_id)
        }
        _ => PrePropose::default().query(deps, env, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, PreProposeOverruleError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
