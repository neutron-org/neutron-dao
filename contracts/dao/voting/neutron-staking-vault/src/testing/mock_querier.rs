use std::collections::HashMap;

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, Binary, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, StdError, StdResult, SystemError, SystemResult, Uint128,
    WasmQuery,
};
use neutron_staking_info_proxy_common::query::ProviderStakeQueryMsg;

pub const MOCK_STAKING_TRACKER: &str = "neutronmockstakingtracker";

pub fn mock_dependencies_staking() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(MockQuerier::new(&[]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: std::marker::PhantomData,
    }
}

pub struct WasmMockQuerier {
    // base: MockQuerier,
    pub stake: HashMap<String, Uint128>,
}

impl WasmMockQuerier {
    fn new(_base: MockQuerier) -> Self {
        WasmMockQuerier {
            // base,
            stake: HashMap::new(),
        }
    }
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

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match contract_addr.as_str() {
                    MOCK_STAKING_TRACKER => {
                        let q: ProviderStakeQueryMsg = from_json(msg).unwrap();
                        let resp: StdResult<Binary> = match q {
                            ProviderStakeQueryMsg::StakeAtHeight { address, height: _ } => {
                                if let Some(stake) = self.stake.get(&address) {
                                    to_json_binary(stake)
                                } else {
                                    Err(StdError::generic_err("no stake for user"))
                                }
                            }
                            ProviderStakeQueryMsg::TotalStakeAtHeight { .. } => to_json_binary(
                                &self.stake.values().fold(Uint128::zero(), |acc, b| acc + b),
                            ),
                        };
                        SystemResult::Ok(ContractResult::from(resp))
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }

    /// Allows setting mock stake for testing.
    pub fn with_stake(&mut self, addr: &Addr, stake: Uint128) {
        self.stake.insert(addr.to_string(), stake);
    }
}
