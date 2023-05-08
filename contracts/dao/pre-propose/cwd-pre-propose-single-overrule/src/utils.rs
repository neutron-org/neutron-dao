use crate::contract::PrePropose;
use crate::error::PreProposeOverruleError;
use crate::state::PROPOSALS;
use cosmwasm_std::{Addr, DepsMut, StdError, StdResult};
use cwd_voting::pre_propose::ProposalCreationPolicy;
use neutron_dao_pre_propose_overrule::msg::{
    MainDaoQueryMsg, ProposalSingleQueryMsg, SubDao, SubdaoConfig, SubdaoProposalModule,
    SubdaoQueryMsg,
};
use neutron_subdao_pre_propose_single::msg::{
    QueryExt as SubdaoPreProposeQueryExt, QueryMsg as SubdaoPreProposeQueryMsg,
};
use neutron_subdao_proposal_single::msg as SubdaoProposalMsg;
use neutron_subdao_timelock_single::{msg as TimelockMsg, types as TimelockTypes};

pub(crate) fn get_subdao_from_timelock(
    deps: &DepsMut,
    timelock_contract: &Addr,
) -> Result<Addr, PreProposeOverruleError> {
    let timelock_config: TimelockTypes::Config = deps.querier.query_wasm_smart(
        timelock_contract.to_string(),
        &TimelockMsg::QueryMsg::Config {},
    )?;
    Ok(timelock_config.subdao)
}

fn query_proposal_modules(
    subdao_core: &Addr,
    deps: &DepsMut,
) -> StdResult<Vec<SubdaoProposalModule>> {
    deps.querier.query_wasm_smart(
        subdao_core,
        &SubdaoQueryMsg::ProposalModules {
            start_after: None,
            limit: None,
        },
    )
}

fn query_proposal_creation_policy(
    proposal_module_address: Addr,
    deps: &DepsMut,
) -> StdResult<ProposalCreationPolicy> {
    deps.querier.query_wasm_smart(
        proposal_module_address,
        &SubdaoProposalMsg::QueryMsg::ProposalCreationPolicy {},
    )
}

fn query_timelock_address(addr: Addr, deps: &DepsMut) -> StdResult<Addr> {
    deps.querier.query_wasm_smart::<Addr>(
        addr,
        &SubdaoPreProposeQueryMsg::QueryExtension {
            msg: SubdaoPreProposeQueryExt::TimelockAddress {},
        },
    )
}

fn process_proposal_modules(
    proposal_modules: Vec<SubdaoProposalModule>,
    expected_timelock: Addr,
    deps: &DepsMut,
) -> Result<(), PreProposeOverruleError> {
    for proposal_module in proposal_modules {
        let prop_policy = query_proposal_creation_policy(proposal_module.address.clone(), deps)?;
        if let ProposalCreationPolicy::Module { addr } = prop_policy {
            if let Ok(timelock) = query_timelock_address(addr, deps) {
                if expected_timelock == timelock {
                    if proposal_module.address == expected_timelock {
                        deps.api.debug("Proposal module found");
                        return Err(PreProposeOverruleError::ForbiddenSubdao {});
                    }
                }
            }
        }
    }
    Err(PreProposeOverruleError::SubdaoMisconfigured {})
}

fn verify_is_timelock_from_subdao(
    deps: &DepsMut,
    subdao_core: &Addr,
    expected_timelock: &Addr,
) -> Result<(), PreProposeOverruleError> {
    // Main function
    let proposal_modules = query_proposal_modules(subdao_core, deps)?;
    process_proposal_modules(proposal_modules, expected_timelock.clone(), deps)
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

pub(crate) fn get_next_proposal_id(deps: &DepsMut) -> Result<u64, PreProposeOverruleError> {
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

pub(crate) fn perform_checks(
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

pub(crate) fn get_subdao_name(
    deps: &DepsMut,
    subdao: &Addr,
) -> Result<String, PreProposeOverruleError> {
    let subdao_config: SubdaoConfig = deps
        .querier
        .query_wasm_smart(subdao, &SubdaoQueryMsg::Config {})?;
    Ok(subdao_config.name)
}
