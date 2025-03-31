use std::collections::HashMap;

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, Binary, ContractResult, GrpcQuery, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, Uint128,
};
use neutron_std::types::cosmos::staking::v1beta1::{
    Delegation, DelegationResponse, QueryDelegationRequest, QueryDelegationResponse,
    QueryValidatorDelegationsRequest, QueryValidatorDelegationsResponse, QueryValidatorRequest,
    QueryValidatorResponse, QueryValidatorsRequest, QueryValidatorsResponse, Validator,
};
use prost::Message;

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(MockQuerier::new(&[]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: std::marker::PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier,
    validators: HashMap<String, Validator>,
    pub delegations: HashMap<(String, String), Uint128>, // (delegator, validator) -> shares
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest = match from_json(bin_request) {
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
    pub fn new(base: MockQuerier) -> Self {
        WasmMockQuerier {
            base,
            validators: HashMap::new(),
            delegations: HashMap::new(),
        }
    }

    pub fn handle_query(&self, request: &QueryRequest) -> QuerierResult {
        match request {
            QueryRequest::Grpc(GrpcQuery { path, data }) => match path.as_str() {
                "/cosmos.staking.v1beta1.Query/Validator" => {
                    let request: QueryValidatorRequest = Message::decode(&data[..]).unwrap();
                    self.handle_validator_query(request)
                }
                "/cosmos.staking.v1beta1.Query/Validators" => {
                    let request: QueryValidatorsRequest = Message::decode(&data[..]).unwrap();
                    self.handle_validators_query(request)
                }
                "/cosmos.staking.v1beta1.Query/Delegation" => {
                    let request: QueryDelegationRequest = Message::decode(&data[..]).unwrap();
                    self.handle_delegation_query(request)
                }
                "/cosmos.staking.v1beta1.Query/ValidatorDelegations" => {
                    let request: QueryValidatorDelegationsRequest =
                        Message::decode(&data[..]).unwrap();
                    self.handle_validator_delegations_query(request)
                }
                _ => {
                    println!("❌ Unsupported GRPC Query: {}", path);
                    self.base.handle_query(request)
                }
            },
            _ => self.base.handle_query(request),
        }
    }

    fn handle_validator_query(&self, request: QueryValidatorRequest) -> QuerierResult {
        if let Some(validator) = self.validators.get(&request.validator_addr) {
            let response = QueryValidatorResponse {
                validator: Some(validator.clone()),
            };

            let mut buf = Vec::new();
            if let Err(e) = response.encode(&mut buf) {
                return SystemResult::Err(SystemError::InvalidResponse {
                    error: format!("Failed to encode Protobuf response: {}", e),
                    response: Default::default(),
                });
            }

            SystemResult::Ok(ContractResult::Ok(Binary::from(buf)))
        } else {
            SystemResult::Err(SystemError::InvalidRequest {
                error: format!("Validator not found: {}", request.validator_addr),
                request: Binary::new(Vec::from("")),
            })
        }
    }

    /// Handles a query for **all validators** (optionally filtered by status).
    fn handle_validators_query(&self, request: QueryValidatorsRequest) -> QuerierResult {
        let validators: Vec<Validator> = self
            .validators
            .values()
            .filter(|v| request.status.is_empty() || v.status.to_string() == request.status)
            .cloned()
            .collect();

        let response = QueryValidatorsResponse {
            validators,
            pagination: None, // Mock doesn't handle pagination
        };

        let mut buf = Vec::new();
        if let Err(e) = response.encode(&mut buf) {
            return SystemResult::Err(SystemError::InvalidResponse {
                error: format!("Failed to encode Protobuf response: {}", e),
                response: Default::default(),
            });
        }

        SystemResult::Ok(ContractResult::Ok(Binary::from(buf)))
    }

    fn handle_delegation_query(&self, request: QueryDelegationRequest) -> QuerierResult {
        let key = (
            request.delegator_addr.clone(),
            request.validator_addr.clone(),
        );
        let shares = self
            .delegations
            .get(&key)
            .cloned()
            .unwrap_or(Uint128::zero());

        let response = QueryDelegationResponse {
            delegation_response: Some(DelegationResponse {
                delegation: Some(Delegation {
                    delegator_address: request.delegator_addr,
                    validator_address: request.validator_addr,
                    shares: shares.to_string(),
                }),
                balance: None,
            }),
        };

        let mut buf = Vec::new();
        if let Err(e) = response.encode(&mut buf) {
            return SystemResult::Err(SystemError::InvalidResponse {
                error: format!("Failed to encode Protobuf response: {}", e),
                response: Default::default(),
            });
        }

        SystemResult::Ok(ContractResult::Ok(Binary::from(buf)))
    }

    fn handle_validator_delegations_query(
        &self,
        request: QueryValidatorDelegationsRequest,
    ) -> QuerierResult {
        println!(
            "🔍 Mock Querier received delegation query for validator: {}",
            request.validator_addr
        );

        let delegations: Vec<DelegationResponse> = self
            .delegations
            .iter()
            .filter(|((_, validator_addr), _)| validator_addr == &request.validator_addr)
            .map(|((delegator, validator_addr), shares)| {
                println!(
                    " Found delegation: Delegator: {}, Validator: {}, Shares: {}",
                    delegator, validator_addr, shares
                );

                DelegationResponse {
                    delegation: Some(Delegation {
                        delegator_address: delegator.clone(),
                        validator_address: validator_addr.clone(),
                        shares: shares.to_string(),
                    }),
                    balance: None,
                }
            })
            .collect();

        if delegations.is_empty() {
            println!(
                "❌ No delegations found for validator: {}",
                request.validator_addr
            );
        }

        let response = QueryValidatorDelegationsResponse {
            delegation_responses: delegations,
            pagination: None, // Mock doesn't handle pagination
        };

        //  **Fix: Serialize the response using Protobuf instead of JSON**
        let mut buf = Vec::new();
        if let Err(e) = response.encode(&mut buf) {
            println!("❌ Failed to serialize Protobuf response: {}", e);
            return SystemResult::Err(SystemError::InvalidResponse {
                error: format!("Failed to serialize Protobuf response: {}", e),
                response: Default::default(),
            });
        }

        println!(" Returning properly encoded Protobuf validator delegations response");

        SystemResult::Ok(ContractResult::Ok(Binary::from(buf)))
    }
    /// Allows setting **mock validators** for testing.
    pub fn with_validators(&mut self, validators: Vec<Validator>) {
        self.validators = validators
            .into_iter()
            .map(|v| (v.operator_address.clone(), v))
            .collect();
    }

    /// Allows setting **mock delegations** for testing.
    pub fn with_delegations(&mut self, delegations: HashMap<(String, String), Uint128>) {
        for ((delegator_addr, validator_addr), shares) in delegations.iter() {
            self.delegations
                .insert((delegator_addr.clone(), validator_addr.clone()), *shares);
        }
    }
}
