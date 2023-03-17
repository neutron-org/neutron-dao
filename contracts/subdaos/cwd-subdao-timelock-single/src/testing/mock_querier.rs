use std::marker::PhantomData;

use cosmwasm_std::{
    from_binary, from_slice,
    testing::{MockApi, MockQuerier, MockStorage},
    to_binary, Addr, ContractResult, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, WasmQuery,
};
use cw_utils::Duration;
use cwd_proposal_single::{
    msg::{QueryMsg as ProposeQuery, QueryMsg},
    state::Config as OverrulProposalConfig,
};
use cwd_voting::threshold::Threshold;
use neutron_dao_pre_propose_overrule::msg::{
    QueryExt as PreProposeOverruleQueryExt, QueryMsg as PreProposeOverruleQuery,
};
use neutron_subdao_pre_propose_single::msg::{
    QueryExt as PreProposeQueryExt, QueryMsg as PreProposeQuery,
};

pub const MOCK_SUBDAO_CORE_ADDR: &str = "neutron1subdao_core_contract";
pub const MOCK_TIMELOCK_INITIALIZER: &str = "neutron1timelock_initializer";
pub const MOCK_MAIN_DAO_ADDR: &str = "neutron1main_dao_core_contract";
pub const MOCK_OVERRULE_PROPOSAL: &str = "neutron1main_dao_overrule_proposal";
pub const MOCK_OVERRULE_PREPROPOSAL: &str = "neutron1main_dao_overrule_preproposal";

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier = WasmMockQuerier::new(MockQuerier::new(&[]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return QuerierResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if contract_addr == MOCK_TIMELOCK_INITIALIZER {
                    let q: PreProposeQuery = from_binary(msg).unwrap();
                    let addr = match q {
                        PreProposeQuery::ProposalModule {} => {
                            todo!()
                        }
                        PreProposeQuery::Dao {} => MOCK_SUBDAO_CORE_ADDR,
                        PreProposeQuery::Config {} => todo!(),
                        PreProposeQuery::DepositInfo { proposal_id: _ } => todo!(),
                        PreProposeQuery::QueryExtension {
                            msg: PreProposeQueryExt::TimelockAddress {},
                        } => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(to_binary(addr)));
                }
                if contract_addr == MOCK_SUBDAO_CORE_ADDR {
                    let addr = { MOCK_MAIN_DAO_ADDR };
                    return SystemResult::Ok(ContractResult::from(to_binary(addr)));
                }
                if contract_addr == MOCK_OVERRULE_PREPROPOSAL {
                    let q: PreProposeOverruleQuery = from_binary(msg).unwrap();
                    let reply = match q {
                        PreProposeOverruleQuery::ProposalModule {} => {
                            MOCK_OVERRULE_PROPOSAL.to_string()
                        }
                        PreProposeOverruleQuery::Dao {} => MOCK_MAIN_DAO_ADDR.to_string(),
                        PreProposeOverruleQuery::Config {} => todo!(),
                        PreProposeOverruleQuery::DepositInfo { proposal_id: _ } => todo!(),
                        PreProposeOverruleQuery::QueryExtension {
                            msg: PreProposeOverruleQueryExt::OverruleProposalId { .. },
                        } => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(to_binary(&reply)));
                }
                if contract_addr == MOCK_OVERRULE_PROPOSAL {
                    let q: ProposeQuery = from_binary(msg).unwrap();
                    let reply = match q {
                        QueryMsg::Config {} => OverrulProposalConfig {
                            threshold: Threshold::AbsoluteCount {
                                threshold: Default::default(),
                            },
                            max_voting_period: Duration::Time(10),
                            min_voting_period: None,
                            allow_revoting: false,
                            dao: Addr::unchecked(MOCK_MAIN_DAO_ADDR),
                            close_proposal_on_execution_failure: false,
                        },
                        QueryMsg::Proposal { .. } => todo!(),
                        QueryMsg::ListProposals { .. } => todo!(),
                        QueryMsg::ReverseProposals { .. } => todo!(),
                        QueryMsg::ProposalCount { .. } => todo!(),
                        QueryMsg::GetVote { .. } => todo!(),
                        QueryMsg::ListVotes { .. } => todo!(),
                        QueryMsg::ProposalCreationPolicy { .. } => todo!(),
                        QueryMsg::ProposalHooks { .. } => todo!(),
                        QueryMsg::VoteHooks { .. } => todo!(),
                        _ => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(to_binary(&reply)));
                }
                SystemResult::Err(SystemError::NoSuchContract {
                    addr: contract_addr.to_string(),
                })
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier) -> Self {
        WasmMockQuerier { base }
    }
}
