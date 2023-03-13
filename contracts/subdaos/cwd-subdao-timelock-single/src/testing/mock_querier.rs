use std::marker::PhantomData;

use cosmwasm_std::{
    from_binary, from_slice,
    testing::{MockApi, MockQuerier, MockStorage},
    to_binary, ContractResult, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult, WasmQuery,
};
use cwd_pre_propose_base::msg::QueryMsg as PreProposeQueryBase;
use neutron_subdao_pre_propose_single::msg::QueryMsg as PreProposeQuery;

pub const MOCK_SUBDAO_CORE_ADDR: &str = "neutron1subdao_core_contract";
pub const MOCK_TIMELOCK_INITIALIZER: &str = "neutron1timelock_initializer";
pub const MOCK_MAIN_DAO_ADDR: &str = "neutron1main_dao_core_contract";

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
                        PreProposeQuery::QueryBase(PreProposeQueryBase::ProposalModule {}) => {
                            todo!()
                        }
                        PreProposeQuery::QueryBase(PreProposeQueryBase::Dao {}) => {
                            MOCK_SUBDAO_CORE_ADDR
                        }
                        PreProposeQuery::QueryBase(PreProposeQueryBase::Config {}) => todo!(),
                        PreProposeQuery::QueryBase(PreProposeQueryBase::DepositInfo {
                            proposal_id: _,
                        }) => todo!(),
                        PreProposeQuery::TimelockAddress {} => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(to_binary(addr)));
                }
                if contract_addr == MOCK_SUBDAO_CORE_ADDR {
                    let addr = { MOCK_MAIN_DAO_ADDR };
                    return SystemResult::Ok(ContractResult::from(to_binary(addr)));
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
