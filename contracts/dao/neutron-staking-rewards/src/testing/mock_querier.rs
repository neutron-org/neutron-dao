use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    coin, from_json, to_json_binary, Binary, Coin, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, StdResult, SystemError, SystemResult, WasmQuery,
};
use neutron_staking_info_proxy_common::msg::QueryMsg as InfoProxyQueryMsg;
use std::collections::HashMap;
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
    pub user_balances: HashMap<String, Vec<(u64, Coin)>>,
}

impl WasmMockQuerier {
    pub fn update_stake(&mut self, user: String, height: u64, amount: Coin) {
        self.user_balances
            .entry(user)
            .or_default()
            .push((height, amount));
    }

    // to update last amount to other value (this can happen since multiple actions can be called in one block)
    pub fn update_last_stake(&mut self, user: String, amount: Coin) {
        let (height, _) = self
            .user_balances
            .entry(user.clone())
            .or_default()
            .pop()
            .unwrap();
        self.update_stake(user, height, amount);
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

const STAKING_INFO_PROXY_CONTRACT: &str =
    "neutron1zfqexm2d6w4ddrl8h77lap2tjdvjd0r83lrjxalp29nq0zgkyfaq629dj9";

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match contract_addr.as_str() {
                    STAKING_INFO_PROXY_CONTRACT => {
                        let q: InfoProxyQueryMsg = from_json(msg).unwrap();
                        let resp: StdResult<Binary> = match q {
                            InfoProxyQueryMsg::UserStake { address, height } => {
                                let balance_history = self.user_balances.get(&address).unwrap();
                                let mut result = to_json_binary(&coin(0u128, "untrn"));
                                for (historical_height, amount) in balance_history.iter().rev() {
                                    if height >= *historical_height {
                                        result = to_json_binary(amount);
                                        break;
                                    }
                                }
                                println!(
                                    "UserStake query to height={:?} result={:?}",
                                    height, result
                                );
                                result
                            }
                            _ => unimplemented!(),
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
        WasmMockQuerier {
            base,
            user_balances: Default::default(),
        }
    }
}
