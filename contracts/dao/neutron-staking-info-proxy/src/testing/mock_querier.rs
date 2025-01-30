use crate::msg::ProviderStakeQuery;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, to_json_binary, Binary, ContractResult, Empty, OwnedDeps, Querier, QuerierResult,
    QueryRequest, StdResult, SystemError, SystemResult, WasmQuery,
};
use std::marker::PhantomData;

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_storage = MockStorage::default();
    let custom_querier = WasmMockQuerier::new(MockQuerier::new(&[]));

    OwnedDeps {
        storage: custom_storage,
        api: MockApi::default().with_prefix("neutron"),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_json(bin_request) {
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

pub const STAKING_REWARDS_CONTRACT: &str =
    "neutron1zfqexm2d6w4ddrl8h77lap2tjdvjd0r83lrjxalp29nq0zgkyfaq629dj9";
pub const PROVIDER1: &str = "neutron173ngx9yztcjyr40nay83qwee6hsvrjzz0ahn97cjug2ckdzaz7lswtwnqh";
pub const PROVIDER2: &str = "neutron15nxt28yhceft6k32zk87mvdnk7qact5uj4q8cc26ldmqzpav2txq4dda03";

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match contract_addr.as_str() {
                    STAKING_REWARDS_CONTRACT => {
                        let q: ProviderStakeQuery = from_json(msg).unwrap();
                        let resp: StdResult<Binary> = match q {
                            ProviderStakeQuery::User { address: _ } => to_json_binary(&1),
                        };
                        SystemResult::Ok(ContractResult::from(resp))
                    }
                    _ => todo!(),
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    fn new(base: MockQuerier) -> WasmMockQuerier {
        WasmMockQuerier { base }
    }
}
