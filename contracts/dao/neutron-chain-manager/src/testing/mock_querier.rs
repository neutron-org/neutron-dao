use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, Binary, ContractResult, Empty, GrpcQuery, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult,
};
use neutron_std::types::cosmos::base::v1beta1::Coin;
use neutron_std::types::cosmos::base::v1beta1::DecCoin;
use neutron_std::types::gaia::globalfee;
use neutron_std::types::neutron::{cron, dex, dynamicfees};
use neutron_std::types::osmosis::tokenfactory;
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
            QueryRequest::Grpc(GrpcQuery { data: _, path }) => match path.as_str() {
                cron::QueryParamsRequest::PATH => {
                    let resp = cron::QueryParamsResponse {
                        params: Some(cron::Params {
                            security_address: "neutron_dao_address".to_string(),
                            limit: 10,
                        }),
                    }
                    .to_proto_bytes();
                    SystemResult::Ok(ContractResult::Ok(Binary::new(resp.to_vec())))
                }
                tokenfactory::v1beta1::QueryParamsRequest::PATH => {
                    let resp = &tokenfactory::v1beta1::QueryParamsResponse {
                        params: Some(tokenfactory::Params {
                            denom_creation_fee: vec![Coin {
                                denom: "untrn".to_string(),
                                amount: "1".to_string(),
                            }],
                            denom_creation_gas_consume: None,
                            fee_collector_address: "test_addr".to_string(),
                            whitelisted_hooks: vec![],
                            track_before_send_gas_limit: 123456,
                        }),
                    }
                    .to_proto_bytes();
                    SystemResult::Ok(ContractResult::Ok(Binary::new(resp.to_vec())))
                }
                dex::QueryParamsRequest::PATH => {
                    let resp = &dex::QueryParamsResponse {
                        params: Some(dex::Params {
                            fee_tiers: [1, 2, 99].to_vec(),
                            paused: false,
                            max_jits_per_block: 20,
                            good_til_purge_allowance: 25000,
                            whitelisted_lps: vec![],
                        }),
                    }
                    .to_proto_bytes();
                    SystemResult::Ok(ContractResult::Ok(Binary::new(resp.to_vec())))
                }
                dynamicfees::v1::QueryParamsRequest::PATH => {
                    let resp = &dynamicfees::v1::QueryParamsResponse {
                        params: Some(dynamicfees::v1::Params {
                            ntrn_prices: vec![DecCoin {
                                denom: "uatom".to_string(),
                                amount: "0.5".to_string(),
                            }],
                        }),
                    }
                    .to_proto_bytes();
                    SystemResult::Ok(ContractResult::Ok(Binary::new(resp.to_vec())))
                }
                globalfee::v1beta1::QueryParamsRequest::PATH => {
                    let resp = &globalfee::v1beta1::QueryParamsResponse {
                        params: Some(default_globalfee_params()),
                    }
                    .to_proto_bytes();
                    SystemResult::Ok(ContractResult::Ok(Binary::new(resp.to_vec())))
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

pub fn default_globalfee_params() -> globalfee::v1beta1::Params {
    globalfee::v1beta1::Params {
        minimum_gas_prices: vec![
            DecCoin {
                denom: "untrn".to_string(),
                amount: "0.025".to_string(),
            },
            DecCoin {
                denom: "uatom".to_string(),
                amount: "0.0025".to_string(),
            },
        ],
        bypass_min_fee_msg_types: vec!["allowedMsgType".to_string()],
        max_total_bypass_min_fee_msg_gas_usage: 10000,
    }
}
