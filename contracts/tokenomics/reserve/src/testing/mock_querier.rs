use std::marker::PhantomData;

use cosmwasm_std::{
    from_json,
    testing::{MockApi, MockQuerier, MockStorage},
    to_json_binary, Binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult,
};
use neutron_sdk::{
    bindings::query::NeutronQuery, query::total_burned_neutrons::TotalBurnedNeutronsAmountResponse,
};

const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<NeutronQuery>,
    total_burned_neutrons: Binary,
    throw_error: bool,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<NeutronQuery> = match from_json(bin_request) {
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
    pub fn new(base: MockQuerier<NeutronQuery>) -> Self {
        WasmMockQuerier {
            base,
            total_burned_neutrons: to_json_binary(&Vec::<Coin>::with_capacity(0)).unwrap(),
            throw_error: false,
        }
    }

    pub fn handle_query(&self, request: &QueryRequest<NeutronQuery>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(NeutronQuery::TotalBurnedNeutronsAmount {}) => {
                if self.throw_error {
                    return SystemResult::Ok(ContractResult::Err("Contract error".to_string()));
                }
                SystemResult::Ok(ContractResult::Ok(self.total_burned_neutrons.clone()))
            }
            _ => self.base.handle_query(request),
        }
    }

    pub fn set_total_burned_neutrons(&mut self, coin: Coin) {
        self.total_burned_neutrons =
            to_json_binary(&TotalBurnedNeutronsAmountResponse { coin }).unwrap()
    }

    pub fn set_total_burned_neutrons_error(&mut self, error_state: bool) {
        self.throw_error = error_state
    }
}
