use crate::cron_module_types::{ParamsCron, ParamsResponseCron};
use crate::dex_module_param_types::{ParamsDex, ParamsResponseDex};
use crate::tokenfactory_module_param_types::{ParamsResponseTokenfactory, ParamsTokenfactory};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    coin, from_json, to_json_binary, ContractResult, Empty, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult,
};
use std::marker::PhantomData;

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_storage = MockStorage::default();
    let custom_querier = WasmMockQuerier::new(MockQuerier::new(&[]));

    OwnedDeps {
        storage: custom_storage,
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
            #[allow(deprecated)]
            QueryRequest::Stargate { path, data: _ } => match path.as_str() {
                "/neutron.cron.Query/Params" => {
                    let resp = to_json_binary(&ParamsResponseCron {
                        params: ParamsCron {
                            security_address: "neutron_dao_address".to_string(),
                            limit: 10,
                        },
                    });
                    SystemResult::Ok(ContractResult::from(resp))
                }
                "/osmosis.tokenfactory.v1beta1.Query/Params" => {
                    let resp = to_json_binary(&ParamsResponseTokenfactory {
                        params: ParamsTokenfactory {
                            denom_creation_fee: vec![coin(1, "untrn")],
                            denom_creation_gas_consume: 0,
                            fee_collector_address: "test_addr".to_string(),
                            whitelisted_hooks: vec![],
                        },
                    });
                    SystemResult::Ok(ContractResult::from(resp))
                }
                "/neutron.dex.Query/Params" => {
                    let resp = to_json_binary(&ParamsResponseDex {
                        params: ParamsDex {
                            fee_tiers: [1, 2, 99].to_vec(),
                            paused: false,
                            max_jits_per_block: 20,
                            good_til_purge_allowance: 25000,
                        },
                    });
                    SystemResult::Ok(ContractResult::from(resp))
                }
                _ => todo!(),
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    fn new(base: MockQuerier) -> WasmMockQuerier {
        WasmMockQuerier { base }
    }
}
