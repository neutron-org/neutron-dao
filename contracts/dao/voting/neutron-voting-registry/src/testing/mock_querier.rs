use cosmwasm_std::StdResult;
use cosmwasm_std::{
    from_json,
    testing::{MockApi, MockQuerier, MockStorage},
    to_json_binary, Binary, ContractResult, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, Uint128, WasmQuery,
};
use cwd_interface::voting::{TotalPowerAtHeightResponse, VotingPowerAtHeightResponse};
use neutron_vault::msg::QueryMsg as VaultQueryMsg;
use std::marker::PhantomData;

pub const MOCK_VAULT_1: &str = "neutron1votingvault1";
pub const MOCK_VAULT_1_NAME: &str = "voting vault 1";
pub const MOCK_VAULT_1_DESC: &str = "voting vault 1 desc";

pub const MOCK_VAULT_2: &str = "neutron1votingvault2";
pub const MOCK_VAULT_2_NAME: &str = "voting vault 2";
pub const MOCK_VAULT_2_DESC: &str = "voting vault 2 desc";

pub const MOCK_VAULT_3: &str = "neutron1votingvault3";
pub const MOCK_VAULT_3_NAME: &str = "voting vault 3";
pub const MOCK_VAULT_3_DESC: &str = "voting vault 3 desc";

pub const MOCK_VAULT_MEMBER: &str = "neutron1member";
pub const MOCK_VAULT_1_VP: u128 = 100u128;
pub const MOCK_VAULT_2_VP: u128 = 150u128;
pub const MOCK_VAULT_3_VP: u128 = 200u128;

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


impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match contract_addr.as_str() {
                    MOCK_VAULT_1 => {
                        let q: VaultQueryMsg = from_json(msg).unwrap();
                        let resp: StdResult<Binary> = match q {
                            VaultQueryMsg::VotingPowerAtHeight { address, height } => {
                                if address.as_str() == MOCK_VAULT_MEMBER {
                                    to_json_binary(&VotingPowerAtHeightResponse {
                                        power: Uint128::from(MOCK_VAULT_1_VP),
                                        height: height.unwrap_or_default(),
                                    })
                                } else {
                                    to_json_binary(&VotingPowerAtHeightResponse {
                                        power: Uint128::zero(),
                                        height: height.unwrap_or_default(),
                                    })
                                }
                            }
                            VaultQueryMsg::TotalPowerAtHeight { height } => {
                                to_json_binary(&TotalPowerAtHeightResponse {
                                    power: Uint128::from(MOCK_VAULT_1_VP),
                                    height: height.unwrap_or_default(),
                                })
                            }
                            VaultQueryMsg::Name {} => {
                                to_json_binary(&String::from(MOCK_VAULT_1_NAME))
                            }
                            VaultQueryMsg::Description {} => {
                                to_json_binary(&String::from(MOCK_VAULT_1_DESC))
                            }
                            _ => todo!(),
                        };
                        SystemResult::Ok(ContractResult::from(resp))
                    }
                    MOCK_VAULT_2 => {
                        let q: VaultQueryMsg = from_json(msg).unwrap();
                        let resp: StdResult<Binary> = match q {
                            VaultQueryMsg::VotingPowerAtHeight { address, height } => {
                                if address.as_str() == MOCK_VAULT_MEMBER {
                                    to_json_binary(&VotingPowerAtHeightResponse {
                                        power: Uint128::from(MOCK_VAULT_2_VP),
                                        height: height.unwrap_or_default(),
                                    })
                                } else {
                                    to_json_binary(&VotingPowerAtHeightResponse {
                                        power: Uint128::zero(),
                                        height: height.unwrap_or_default(),
                                    })
                                }
                            }
                            VaultQueryMsg::TotalPowerAtHeight { height } => {
                                to_json_binary(&TotalPowerAtHeightResponse {
                                    power: Uint128::from(MOCK_VAULT_2_VP),
                                    height: height.unwrap_or_default(),
                                })
                            }
                            VaultQueryMsg::Name {} => {
                                to_json_binary(&String::from(MOCK_VAULT_2_NAME))
                            }
                            VaultQueryMsg::Description {} => {
                                to_json_binary(&String::from(MOCK_VAULT_2_DESC))
                            }
                            _ => todo!(),
                        };
                        SystemResult::Ok(ContractResult::from(resp))
                    }
                    MOCK_VAULT_3 => {
                        let q: VaultQueryMsg = from_json(msg).unwrap();
                        let resp: StdResult<Binary> = match q {
                            VaultQueryMsg::VotingPowerAtHeight { address, height } => {
                                if address.as_str() == MOCK_VAULT_MEMBER {
                                    to_json_binary(&VotingPowerAtHeightResponse {
                                        power: Uint128::from(MOCK_VAULT_3_VP),
                                        height: height.unwrap_or_default(),
                                    })
                                } else {
                                    to_json_binary(&VotingPowerAtHeightResponse {
                                        power: Uint128::zero(),
                                        height: height.unwrap_or_default(),
                                    })
                                }
                            }
                            VaultQueryMsg::TotalPowerAtHeight { height } => {
                                to_json_binary(&TotalPowerAtHeightResponse {
                                    power: Uint128::from(MOCK_VAULT_3_VP),
                                    height: height.unwrap_or_default(),
                                })
                            }
                            VaultQueryMsg::Name {} => {
                                to_json_binary(&String::from(MOCK_VAULT_3_NAME))
                            }
                            VaultQueryMsg::Description {} => {
                                to_json_binary(&String::from(MOCK_VAULT_3_DESC))
                            }
                            _ => todo!(),
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
