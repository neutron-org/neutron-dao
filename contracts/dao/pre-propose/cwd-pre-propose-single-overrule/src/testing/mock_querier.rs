use std::marker::PhantomData;

// use crate::contract::{
//     MainDaoQueryMsg, ProposalStatus, SingleChoiceProposal, SubDao, TimelockConfig, TimelockQueryMsg,
// };
use cosmwasm_std::{
    from_binary, from_slice,
    testing::{MockApi, MockQuerier, MockStorage},
    to_binary, Addr, ContractResult, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, WasmQuery,
};
use cwd_core::{msg::QueryMsg as MainDaoQueryMsg, query::SubDao};
use cwd_proposal_single::msg::QueryMsg as ProposalSingleQueryMsg;

use neutron_dao_pre_propose_overrule::msg::{
    ExecuteMsg, InstantiateMsg, ProposeMessageInternal, QueryMsg,
};
use neutron_subdao_core::{msg::QueryMsg as SubdaoQueryMsg, types as SubdaoTypes};
use neutron_subdao_pre_propose_single::msg::QueryMsg as SubdaoPreProposeQueryMsg;
use neutron_subdao_proposal_single::msg as SubdaoProposalMsg;
use neutron_subdao_timelock_single::types::{ProposalStatus, SingleChoiceProposal};
use neutron_subdao_timelock_single::{msg as TimelockMsg, types as TimelockTypes};

pub const MOCK_DAO_CORE: &str = "neutron1dao_core_contract";
pub const MOCK_DAO_CORE_MANY_SUBDAOS: &str = "neutron1dao_core_contract_many_subdaos";
pub const MOCK_SUBDAO_PROPOSE_MODULE: &str = "neutron1subdao_propose_module";
pub const MOCK_SUBDAO_PREPROPOSE_MODULE: &str = "neutron1subdao_prepropose_module";
pub const MOCK_DAO_PROPOSE_MODULE: &str = "neutron1propose_module";
pub const MOCK_TIMELOCK_CONTRACT: &str = "neutron1timelock_contract";
pub const MOCK_SUBDAO_CORE: &str = "neutron1subdao_core";
pub const SUBDAO_NAME: &str = "Based DAO";

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
                    error: format!("Parsing query request: {:?}", e),
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
                if contract_addr == MOCK_SUBDAO_PROPOSE_MODULE {
                    let q: ProposalSingleQueryMsg = from_binary(msg).unwrap();
                    return match q {
                        ProposalSingleQueryMsg::Dao {} => {
                            SystemResult::Ok(ContractResult::from(to_binary(MOCK_DAO_CORE)))
                        }
                        ProposalSingleQueryMsg::ProposalCount {} => {
                            SystemResult::Ok(ContractResult::from(to_binary(&(0 as u64))))
                        }
                        _ => SystemResult::Err(SystemError::Unknown {}),
                    };
                }
                if contract_addr == MOCK_DAO_CORE {
                    let q: MainDaoQueryMsg = from_binary(msg).unwrap();
                    return match q {
                        MainDaoQueryMsg::ListSubDaos {
                            start_after: _,
                            limit: _,
                        } => SystemResult::Ok(ContractResult::from(to_binary(&vec![SubDao {
                            addr: MOCK_SUBDAO_CORE.to_string(),
                            charter: None,
                        }]))),
                        _ => SystemResult::Err(SystemError::Unknown {}),
                    };
                }
                if contract_addr == MOCK_TIMELOCK_CONTRACT {
                    let q: TimelockMsg::QueryMsg = from_binary(msg).unwrap();
                    return match q {
                        TimelockMsg::QueryMsg::Config {} => SystemResult::Ok(ContractResult::from(
                            to_binary(&TimelockTypes::Config {
                                owner: Addr::unchecked(MOCK_DAO_CORE),
                                overrule_pre_propose: Addr::unchecked(""),
                                subdao: Addr::unchecked(MOCK_SUBDAO_CORE),
                            }),
                        )),
                        TimelockMsg::QueryMsg::Proposal { proposal_id } => SystemResult::Ok(
                            ContractResult::from(to_binary(&SingleChoiceProposal {
                                id: proposal_id,
                                timelock_ts: Default::default(),
                                msgs: vec![],
                                status: ProposalStatus::Timelocked,
                            })),
                        ),
                        _ => SystemResult::Err(SystemError::Unknown {}),
                    };
                }
                if contract_addr == MOCK_SUBDAO_CORE {
                    let q: SubdaoQueryMsg = from_binary(msg).unwrap();
                    return match q {
                        SubdaoQueryMsg::ProposalModules {
                            start_after: _,
                            limit: _,
                        } => SystemResult::Ok(ContractResult::from(to_binary(&vec![
                            SubdaoTypes::ProposalModule {
                                address: Addr::unchecked(MOCK_DAO_PROPOSE_MODULE),
                                prefix: "".to_string(),
                                status: SubdaoTypes::ProposalModuleStatus::Enabled,
                            },
                        ]))),
                        SubdaoQueryMsg::Config {} => SystemResult::Ok(ContractResult::from(
                            to_binary(&SubdaoTypes::Config {
                                name: SUBDAO_NAME.to_string(),
                                description: "".to_string(),
                                dao_uri: None,
                                main_dao: Addr::unchecked(MOCK_DAO_CORE),
                                security_dao: Addr::unchecked(""),
                            }),
                        )),
                        _ => SystemResult::Err(SystemError::Unknown {}),
                    };
                }
                if contract_addr == MOCK_DAO_PROPOSE_MODULE {
                    let q: SubdaoProposalMsg::QueryMsg = from_binary(msg).unwrap();
                    return match q {
                        SubdaoProposalMsg::QueryMsg::ProposalCreationPolicy {} => {
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &cwd_voting::pre_propose::ProposalCreationPolicy::Module {
                                    addr: Addr::unchecked(MOCK_SUBDAO_PREPROPOSE_MODULE),
                                },
                            )))
                        }
                        _ => SystemResult::Err(SystemError::Unknown {}),
                    };
                }
                if contract_addr == MOCK_SUBDAO_PREPROPOSE_MODULE {
                    let q: SubdaoPreProposeQueryMsg = from_binary(msg).unwrap();
                    return match q {
                        SubdaoPreProposeQueryMsg::TimelockAddress {} => {
                            SystemResult::Ok(ContractResult::from(to_binary(&Addr::unchecked(
                                MOCK_TIMELOCK_CONTRACT,
                            ))))
                        }
                        _ => SystemResult::Err(SystemError::Unknown {}),
                    };
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
