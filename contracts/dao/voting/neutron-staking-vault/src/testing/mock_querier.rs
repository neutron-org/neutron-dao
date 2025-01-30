use std::collections::HashMap;
use cosmwasm_std::{
    to_json_binary, ContractResult, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, Uint128,
};
use cosmwasm_std::testing::{MockApi, MockStorage};
use neutron_std::types::cosmos::staking::v1beta1::{
    QueryDelegationRequest, QueryDelegationResponse, QueryValidatorRequest, QueryValidatorResponse,
    QueryValidatorsRequest, QueryValidatorsResponse, QueryValidatorDelegationsRequest, QueryValidatorDelegationsResponse,
    Validator, DelegationResponse, Delegation,
};

pub struct WasmMockQuerier {
    delegations: HashMap<(String, String), Uint128>, // (delegator, validator) -> shares
    validators: HashMap<String, Validator>,         // validator address -> Validator
}

impl WasmMockQuerier {
    pub fn new() -> Self {
        Self {
            delegations: HashMap::new(),
            validators: HashMap::new(),
        }
    }

    /// Adds delegations for multiple delegators to a validator.
    pub fn with_validator_delegations(&mut self, validator_addr: &str, delegations: Vec<(String, Uint128)>) {
        for (delegator, shares) in delegations {
            self.delegations.insert((delegator, validator_addr.to_string()), shares);
        }
    }

    pub fn with_delegations(&mut self, delegations: HashMap<(String, String), Uint128>) {
        for ((delegator_addr, validator_addr), shares) in delegations {
            self.delegations.insert((delegator_addr, validator_addr), shares);
        }
    }

    pub fn with_validators(&mut self, validators: Vec<Validator>) {
        self.validators = validators
            .into_iter()
            .map(|v| (v.operator_address.clone(), v))
            .collect();
    }

    fn handle_stargate_query(&self, path: &str, data: &[u8]) -> QuerierResult {
        match path {
            // Handle /cosmos.staking.v1beta1.Query/Validator
            "/cosmos.staking.v1beta1.Query/Validator" => {
                let request: QueryValidatorRequest = match prost::Message::decode(data) {
                    Ok(req) => req,
                    Err(_) => {
                        return SystemResult::Err(SystemError::InvalidRequest {
                            error: "Failed to decode QueryValidatorRequest".to_string(),
                            request: data.into(),
                        });
                    }
                };

                if let Some(validator) = self.validators.get(&request.validator_addr) {
                    let response = QueryValidatorResponse {
                        validator: Some(validator.clone()),
                    };

                    match to_json_binary(&response) {
                        Ok(binary) => SystemResult::Ok(ContractResult::Ok(binary)),
                        Err(e) => SystemResult::Err(SystemError::InvalidResponse {
                            error: format!("Failed to serialize response: {}", e),
                            response: Default::default(),
                        }),
                    }
                } else {
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Validator not found: {}", request.validator_addr),
                        request: data.into(),
                    })
                }
            }

            // Handle /cosmos.staking.v1beta1.Query/Validators
            "/cosmos.staking.v1beta1.Query/Validators" => {
                let request: QueryValidatorsRequest = match prost::Message::decode(data) {
                    Ok(req) => req,
                    Err(_) => {
                        return SystemResult::Err(SystemError::InvalidRequest {
                            error: "Failed to decode QueryValidatorsRequest".to_string(),
                            request: data.into(),
                        });
                    }
                };

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

                match to_json_binary(&response) {
                    Ok(binary) => SystemResult::Ok(ContractResult::Ok(binary)),
                    Err(e) => SystemResult::Err(SystemError::InvalidResponse {
                        error: format!("Failed to serialize response: {}", e),
                        response: Default::default(),
                    }),
                }
            }

            // Handle /cosmos.staking.v1beta1.Query/Delegation
            "/cosmos.staking.v1beta1.Query/Delegation" => {
                let request: QueryDelegationRequest = match prost::Message::decode(data) {
                    Ok(req) => req,
                    Err(_) => {
                        return SystemResult::Err(SystemError::InvalidRequest {
                            error: "Failed to decode QueryDelegationRequest".to_string(),
                            request: data.into(),
                        });
                    }
                };

                let key = (request.delegator_addr.clone(), request.validator_addr.clone());
                let shares = self.delegations.get(&key).cloned().unwrap_or(Uint128::zero());

                let response = QueryDelegationResponse {
                    delegation_response: Some(DelegationResponse {
                        delegation: Some(Delegation {
                            delegator_address: request.delegator_addr,
                            validator_address: request.validator_addr,
                            shares: shares.to_string(),
                            ..Default::default()
                        }),
                        balance: None,
                    }),
                };

                match to_json_binary(&response) {
                    Ok(binary) => SystemResult::Ok(ContractResult::Ok(binary)),
                    Err(e) => SystemResult::Err(SystemError::InvalidResponse {
                        error: format!("Failed to serialize response: {}", e),
                        response: Default::default(),
                    }),
                }
            }

            // Handle /cosmos.staking.v1beta1.Query/ValidatorDelegations
            "/cosmos.staking.v1beta1.Query/ValidatorDelegations" => {
                let request: QueryValidatorDelegationsRequest = match prost::Message::decode(data) {
                    Ok(req) => req,
                    Err(_) => {
                        return SystemResult::Err(SystemError::InvalidRequest {
                            error: "Failed to decode QueryValidatorDelegationsRequest".to_string(),
                            request: data.into(),
                        });
                    }
                };

                let delegations: Vec<DelegationResponse> = self
                    .delegations
                    .iter()
                    .filter(|((_, validator_addr), _)| validator_addr == &request.validator_addr)
                    .map(|((delegator, validator_addr), shares)| DelegationResponse {
                        delegation: Some(Delegation {
                            delegator_address: delegator.clone(),
                            validator_address: validator_addr.clone(),
                            shares: shares.to_string(),
                            ..Default::default()
                        }),
                        balance: None,
                    })
                    .collect();

                let response = QueryValidatorDelegationsResponse {
                    delegation_responses: delegations,
                    pagination: None, // Mock doesn't handle pagination
                };

                match to_json_binary(&response) {
                    Ok(binary) => SystemResult::Ok(ContractResult::Ok(binary)),
                    Err(e) => SystemResult::Err(SystemError::InvalidResponse {
                        error: format!("Failed to serialize response: {}", e),
                        response: Default::default(),
                    }),
                }
            }

            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: format!("Unsupported Stargate path: {}", path),
            }),
        }
    }
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match cosmwasm_std::from_json(bin_request) {
            Ok(r) => r,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Failed to parse QueryRequest: {}", e),
                    request: bin_request.into(),
                });
            }
        };

        match request {
            QueryRequest::Stargate { path, data } => self.handle_stargate_query(&path, &data),
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Unsupported query type".to_string(),
            }),
        }
    }
}

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new();

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: std::marker::PhantomData,
    }
}
