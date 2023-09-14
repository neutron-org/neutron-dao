use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use cosmwasm_std::{
    from_binary, from_slice,
    testing::{MockApi, MockQuerier, MockStorage},
    to_binary, Addr, ContractResult, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, Uint128, WasmQuery,
};
use cw_utils::Duration;
use cwd_proposal_single::{
    msg::QueryMsg as ProposeQuery, proposal::SingleChoiceProposal as MainDaoSingleChoiceProposal,
    query::ProposalResponse as MainDaoProposalResponse, state::Config as OverruleProposalConfig,
};
use cwd_voting::status::Status;
use cwd_voting::threshold::PercentageThreshold::Majority;
use cwd_voting::threshold::Threshold;
use cwd_voting::voting::Votes;
use neutron_dao_pre_propose_overrule::msg::{
    QueryExt as PreProposeOverruleQueryExt, QueryMsg as PreProposeOverruleQuery,
};
use neutron_subdao_core::msg::QueryMsg as CoreSubdaoQuery;
use neutron_subdao_pre_propose_single::msg::{
    QueryExt as PreProposeQueryExt, QueryMsg as PreProposeQuery,
};
use neutron_subdao_proposal_single::types::Config as ProposalConfig;

pub const MOCK_SUBDAO_CORE_ADDR: &str = "neutron1subdao_core_contract";
pub const MOCK_TIMELOCK_INITIALIZER: &str = "neutron1timelock_initializer";
pub const MOCK_MAIN_DAO_ADDR: &str = "neutron1main_dao_core_contract";
pub const MOCK_OVERRULE_PROPOSAL: &str = "neutron1main_dao_overrule_proposal";
pub const MOCK_OVERRULE_PREPROPOSAL: &str = "neutron1main_dao_overrule_preproposal";

pub const MOCK_PROPOSAL_ADDR: &str = "neutron1subdao_proposal_contract";

pub fn mock_dependencies(
    x: Rc<RefCell<Status>>,
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_storage = MockStorage::default();
    let custom_querier = WasmMockQuerier::new(MockQuerier::new(&[]), x);

    OwnedDeps {
        storage: custom_storage,
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier,
    overrule_proposal_status: Rc<RefCell<Status>>,
    close_proposal_on_execution_failure: bool,
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
                    let q: CoreSubdaoQuery = from_binary(msg).unwrap();
                    let addr = match q {
                        CoreSubdaoQuery::MainDao {} => MOCK_MAIN_DAO_ADDR,
                        CoreSubdaoQuery::TimelockProposalModuleAddress { timelock: _ } => {
                            MOCK_PROPOSAL_ADDR
                        }
                        _ => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(to_binary(addr)));
                }
                if contract_addr == MOCK_OVERRULE_PREPROPOSAL {
                    let q: PreProposeOverruleQuery = from_binary(msg).unwrap();
                    let reply = match q {
                        PreProposeOverruleQuery::ProposalModule {} => {
                            to_binary(&MOCK_OVERRULE_PROPOSAL.to_string())
                        }
                        PreProposeOverruleQuery::Dao {} => {
                            to_binary(&MOCK_MAIN_DAO_ADDR.to_string())
                        }
                        PreProposeOverruleQuery::Config {} => todo!(),
                        PreProposeOverruleQuery::DepositInfo { proposal_id: _ } => todo!(),
                        PreProposeOverruleQuery::QueryExtension {
                            msg: PreProposeOverruleQueryExt::OverruleProposalId { .. },
                        } => to_binary(&1),
                    };
                    return SystemResult::Ok(ContractResult::from(reply));
                }
                if contract_addr == MOCK_OVERRULE_PROPOSAL {
                    let q: ProposeQuery = from_binary(msg).unwrap();
                    let reply = match q {
                        ProposeQuery::Config {} => to_binary(&OverruleProposalConfig {
                            threshold: Threshold::AbsoluteCount {
                                threshold: Default::default(),
                            },
                            max_voting_period: Duration::Time(10),
                            min_voting_period: None,
                            allow_revoting: false,
                            dao: Addr::unchecked(MOCK_MAIN_DAO_ADDR),
                            close_proposal_on_execution_failure: false,
                        }),
                        ProposeQuery::Proposal { .. } => to_binary(&MainDaoProposalResponse {
                            id: 1,
                            proposal: MainDaoSingleChoiceProposal {
                                title: "".to_string(),
                                description: "".to_string(),
                                proposer: Addr::unchecked(""),
                                start_height: 0,
                                min_voting_period: None,
                                expiration: Default::default(),
                                threshold: Threshold::AbsoluteCount {
                                    threshold: Uint128::new(1),
                                },
                                total_power: Default::default(),
                                msgs: vec![],
                                // status: Status::Rejected,
                                status: *(*self.overrule_proposal_status).borrow(),
                                votes: Votes {
                                    yes: Default::default(),
                                    no: Default::default(),
                                    abstain: Default::default(),
                                },
                                allow_revoting: false,
                            },
                        }),
                        ProposeQuery::ListProposals { .. } => todo!(),
                        ProposeQuery::ReverseProposals { .. } => todo!(),
                        ProposeQuery::ProposalCount { .. } => todo!(),
                        ProposeQuery::GetVote { .. } => todo!(),
                        ProposeQuery::ListVotes { .. } => todo!(),
                        ProposeQuery::ProposalCreationPolicy { .. } => todo!(),
                        ProposeQuery::ProposalHooks { .. } => todo!(),
                        ProposeQuery::VoteHooks { .. } => todo!(),
                        _ => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(reply));
                }
                if contract_addr == MOCK_PROPOSAL_ADDR {
                    let q: ProposeQuery = from_binary(msg).unwrap();
                    let reply = match q {
                        ProposeQuery::Config {} => to_binary(&ProposalConfig {
                            threshold: Threshold::AbsolutePercentage {
                                percentage: Majority {},
                            },
                            max_voting_period: Duration::Time(1),
                            min_voting_period: None,
                            allow_revoting: false,
                            dao: Addr::unchecked(""),
                            close_proposal_on_execution_failure: self
                                .close_proposal_on_execution_failure,
                        }),
                        _ => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(reply));
                }
                SystemResult::Err(SystemError::NoSuchContract {
                    addr: contract_addr.to_string(),
                })
            }
            _ => self.base.handle_query(request),
        }
    }

    pub fn set_close_proposal_on_execution_failure(&mut self, v: bool) {
        self.close_proposal_on_execution_failure = v
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier, x: Rc<RefCell<Status>>) -> WasmMockQuerier {
        WasmMockQuerier {
            base,
            overrule_proposal_status: x,
            close_proposal_on_execution_failure: true,
        }
    }
}
