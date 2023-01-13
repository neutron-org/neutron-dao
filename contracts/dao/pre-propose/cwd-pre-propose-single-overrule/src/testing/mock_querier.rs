use std::marker::PhantomData;

use crate::contract::{
    MainDaoQueryMsg, ProposalStatus, SingleChoiceProposal, SubDao, TimelockConfig, TimelockQueryMsg,
};
use cosmwasm_std::{
    from_binary, from_slice,
    testing::{MockApi, MockQuerier, MockStorage},
    to_binary, Addr, ContractResult, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, WasmQuery,
};
use cwd_pre_propose_base::msg::QueryMsg as PreProposeQuery;

pub const MOCK_CORE_MODULE: &str = "neutron1dao_core_contract";
pub const MOCK_PROPOSE_MODULE: &str = "neutron1propose_module";
pub const MOCK_TIMELOCK_CONTRACT: &str = "neutron1timelock_contract";
pub const MOCK_SUBDAO_CORE: &str = "neutron1subdao_core";

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
                if contract_addr == MOCK_PROPOSE_MODULE {
                    let q: PreProposeQuery = from_binary(msg).unwrap();
                    let addr = match q {
                        PreProposeQuery::ProposalModule {} => todo!(),
                        PreProposeQuery::Dao {} => MOCK_CORE_MODULE,
                        PreProposeQuery::Config {} => todo!(),
                        PreProposeQuery::DepositInfo { proposal_id: _ } => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(to_binary(addr)));
                }
                if contract_addr == MOCK_CORE_MODULE {
                    let q: MainDaoQueryMsg = from_binary(msg).unwrap();
                    return match q {
                        MainDaoQueryMsg::ListSubDaos { start_after, limit } => {
                            SystemResult::Ok(ContractResult::from(to_binary(&vec![SubDao {
                                addr: MOCK_SUBDAO_CORE.to_string(),
                                charter: None,
                            }])))
                        }
                    };
                }
                if contract_addr == MOCK_TIMELOCK_CONTRACT {
                    let q: TimelockQueryMsg = from_binary(msg).unwrap();
                    return match q {
                        TimelockQueryMsg::Config {} => {
                            SystemResult::Ok(ContractResult::from(to_binary(&TimelockConfig {
                                owner: Addr::unchecked(MOCK_CORE_MODULE),
                                timelock_duration: 0,
                                subdao: Addr::unchecked(MOCK_SUBDAO_CORE),
                            })))
                        }
                        TimelockQueryMsg::Proposal { proposal_id } => SystemResult::Ok(
                            ContractResult::from(to_binary(&SingleChoiceProposal {
                                id: proposal_id,
                                timelock_ts: Default::default(),
                                msgs: vec![],
                                status: ProposalStatus::Timelocked,
                            })),
                        ),
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
