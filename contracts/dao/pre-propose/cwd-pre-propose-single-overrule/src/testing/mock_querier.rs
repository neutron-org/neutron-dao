use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use cosmwasm_std::{
    from_json,
    testing::{MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, Binary, ContractResult, Empty, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, WasmQuery,
};
use cwd_core::msg::QueryMsg as MainDaoQueryMsg;
use cwd_proposal_single::msg::QueryMsg as ProposalSingleQueryMsg;

use neutron_subdao_core::{msg::QueryMsg as SubdaoQueryMsg, types as SubdaoTypes};
use neutron_subdao_timelock_single::types::{ProposalStatus, SingleChoiceProposal};
use neutron_subdao_timelock_single::{msg as TimelockMsg, types as TimelockTypes};

pub const MOCK_DAO_CORE: &str = "neutron1dao_core_contract";
pub const MOCK_DAO_PROPOSE_MODULE: &str = "neutron1propose_module";
pub const MOCK_TIMELOCK_CONTRACT: &str = "neutron1timelock_contract";
pub const MOCK_SUBDAO_CORE: &str = "neutron1subdao_core";

pub const MOCK_IMPOSTOR_TIMELOCK_CONTRACT: &str = "neutron1timelock_contract_impostor";

pub const SUBDAO_NAME: &str = "Based DAO";
pub const TIMELOCKED_PROPOSAL_ID: u64 = 42;
pub const NON_TIMELOCKED_PROPOSAL_ID: u64 = 24;
pub const PROPOSALS_COUNT: u64 = 61;

pub fn mock_dependencies(
    contracts: HashMap<String, Box<dyn ContractQuerier>>,
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier = WasmMockQuerier::new(MockQuerier::new(&[]), contracts);

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier,
    contracts: HashMap<String, Box<dyn ContractQuerier>>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return QuerierResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {:?}", e),
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
                let mock = self.contracts.get(contract_addr);
                match mock {
                    None => SystemResult::Err(SystemError::NoSuchContract {
                        addr: contract_addr.to_string(),
                    }),
                    Some(m) => m.query(msg),
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier, contracts: HashMap<String, Box<dyn ContractQuerier>>) -> Self {
        WasmMockQuerier { base, contracts }
    }
}

pub trait ContractQuerier {
    fn query(&self, msg: &Binary) -> QuerierResult;
}

pub struct MockDaoQueries {
    sub_dao_set: HashSet<String>,
}

impl ContractQuerier for MockDaoQueries {
    fn query(&self, msg: &Binary) -> QuerierResult {
        let q: MainDaoQueryMsg = from_json(msg).unwrap();
        match q {
            MainDaoQueryMsg::GetSubDao { address } => match self.sub_dao_set.contains(&address) {
                true => {
                    SystemResult::Ok(ContractResult::from(to_json_binary(&SubdaoTypes::SubDao {
                        addr: address.clone(),
                        charter: None,
                    })))
                }
                false => SystemResult::Err(SystemError::Unknown {}),
            },
            _ => SystemResult::Err(SystemError::Unknown {}),
        }
    }
}

pub struct MockTimelockQueries {
    owner: String,
    subdao: String,
}

impl ContractQuerier for MockTimelockQueries {
    fn query(&self, msg: &Binary) -> QuerierResult {
        let q: TimelockMsg::QueryMsg = from_json(msg).unwrap();
        match q {
            TimelockMsg::QueryMsg::Config {} => SystemResult::Ok(ContractResult::from(
                to_json_binary(&TimelockTypes::Config {
                    owner: Addr::unchecked(self.owner.clone()),
                    overrule_pre_propose: Addr::unchecked(""),
                    subdao: Addr::unchecked(self.subdao.clone()),
                }),
            )),
            TimelockMsg::QueryMsg::Proposal { proposal_id } => SystemResult::Ok(
                ContractResult::from(to_json_binary(&SingleChoiceProposal {
                    id: proposal_id,
                    msgs: vec![],
                    status: match proposal_id {
                        TIMELOCKED_PROPOSAL_ID => ProposalStatus::Timelocked,
                        _ => ProposalStatus::Executed,
                    },
                })),
            ),
            _ => SystemResult::Err(SystemError::Unknown {}),
        }
    }
}

pub struct MockDaoProposalQueries {
    dao_core: String,
}

impl ContractQuerier for MockDaoProposalQueries {
    fn query(&self, msg: &Binary) -> QuerierResult {
        let q: ProposalSingleQueryMsg = from_json(msg).unwrap();
        match q {
            ProposalSingleQueryMsg::Dao {} => {
                SystemResult::Ok(ContractResult::from(to_json_binary(&self.dao_core)))
            }
            ProposalSingleQueryMsg::ProposalCount {} => {
                SystemResult::Ok(ContractResult::from(to_json_binary(&PROPOSALS_COUNT)))
            }
            _ => SystemResult::Err(SystemError::Unknown {}),
        }
    }
}

pub struct MockSubdaoCoreQueries {
    timelock: String,
    dao_core: String,
}

impl ContractQuerier for MockSubdaoCoreQueries {
    fn query(&self, msg: &Binary) -> QuerierResult {
        let q: SubdaoQueryMsg = from_json(msg).unwrap();
        match q {
            SubdaoQueryMsg::VerifyTimelock { timelock } => SystemResult::Ok(ContractResult::from(
                to_json_binary(&(timelock == self.timelock)),
            )),
            SubdaoQueryMsg::Config {} => {
                SystemResult::Ok(ContractResult::from(to_json_binary(&SubdaoTypes::Config {
                    name: SUBDAO_NAME.to_string(),
                    description: "".to_string(),
                    dao_uri: None,
                    main_dao: Addr::unchecked(self.dao_core.clone()),
                    security_dao: Addr::unchecked(""),
                })))
            }
            _ => SystemResult::Err(SystemError::Unknown {}),
        }
    }
}

pub fn get_properly_initialized_dao() -> HashMap<String, Box<dyn ContractQuerier>> {
    let mut contracts: HashMap<String, Box<dyn ContractQuerier>> = HashMap::new();
    contracts.insert(
        MOCK_DAO_PROPOSE_MODULE.to_string(),
        Box::new(MockDaoProposalQueries {
            dao_core: MOCK_DAO_CORE.parse().unwrap(),
        }),
    );
    contracts.insert(
        MOCK_DAO_CORE.to_string(),
        Box::new(MockDaoQueries {
            sub_dao_set: HashSet::from([MOCK_SUBDAO_CORE.to_string()]),
        }),
    );
    contracts.insert(
        MOCK_TIMELOCK_CONTRACT.to_string(),
        Box::new(MockTimelockQueries {
            owner: MOCK_DAO_CORE.to_string(),
            subdao: MOCK_SUBDAO_CORE.to_string(),
        }),
    );
    contracts.insert(
        MOCK_SUBDAO_CORE.to_string(),
        Box::new(MockSubdaoCoreQueries {
            timelock: MOCK_TIMELOCK_CONTRACT.to_string(),
            dao_core: MOCK_DAO_CORE.to_string(),
        }),
    );
    contracts
}

pub fn get_dao_with_impostor_timelock() -> HashMap<String, Box<dyn ContractQuerier>> {
    let mut contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    // impostor timelock is the same as regular one but the subdao doesn't point to it
    contracts.insert(
        MOCK_IMPOSTOR_TIMELOCK_CONTRACT.to_string(),
        Box::new(MockTimelockQueries {
            owner: MOCK_DAO_CORE.to_string(),
            subdao: MOCK_SUBDAO_CORE.to_string(),
        }),
    );
    contracts
}

pub fn get_dao_with_impostor_subdao() -> HashMap<String, Box<dyn ContractQuerier>> {
    let mut contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    // subdao becomes impostor if it is not in the dao's list, so let's just make it empty
    contracts.remove(&MOCK_DAO_CORE.to_string());
    contracts.insert(
        MOCK_DAO_CORE.to_string(),
        Box::new(MockDaoQueries {
            sub_dao_set: HashSet::from([]),
        }),
    );
    contracts
}
