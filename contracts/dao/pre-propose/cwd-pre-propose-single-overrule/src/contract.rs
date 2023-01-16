#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdResult, Timestamp, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use error::PreProposeOverruleError;
use neutron_bindings::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, ProposeMessage, ProposeMessageInternal, QueryMsg,
    TimelockExecuteMsg,
};
use crate::state::{Config, CONFIG};
use cwd_pre_propose_base::{
    error::PreProposeError,
    msg::{ExecuteMsg as ExecuteBase, InstantiateMsg as InstantiateBase},
    state::PreProposeContract,
};

pub(crate) const CONTRACT_NAME: &str = "crates.io:cwd-pre-propose-single-overrule";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type PrePropose = PreProposeContract<ProposeMessageInternal>;

// EXTERNAL TYPES SECTION BEGIN

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MainDaoQueryMsg {
    ListSubDaos {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubDao {
    /// The contract address of the SubDAO
    pub addr: String,
    /// The purpose/constitution for the SubDAO
    pub charter: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TimelockQueryMsg {
    /// Gets the config. Returns `state::Config`.
    Config {},

    /// Gets information about a proposal. Returns
    /// `proposals::Proposal`.
    Proposal { proposal_id: u64 },
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug, Eq, PartialEq)]
pub struct SingleChoiceProposal {
    /// The ID of the proposal being returned.
    pub id: u64,

    /// The timestamp at which the proposal was submitted to the timelock contract.
    pub timelock_ts: Timestamp,

    /// The messages that will be executed should this proposal be executed.
    pub msgs: Vec<CosmosMsg<NeutronMsg>>,

    pub status: ProposalStatus,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Copy)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ProposalStatus {
    /// The proposal is open for voting.
    Timelocked,
    /// The proposal has been overruled.
    Overruled,
    /// The proposal has been executed.
    Executed,
    /// The proposal's execution failed.
    ExecutionFailed,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct TimelockConfig {
    pub owner: Addr,
    pub timelock_duration: u64,
    // subDAO core module can timelock proposals.
    pub subdao: Addr,
}

// EXTERNAL TYPES SECTION END

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, PreProposeError> {
    // the contract has no info for instantiation so far, so it just calls the init function of base
    // deposit is set to zero because it makes no sense for overrule proposals
    // for open submission it's tbd
    let resp = PrePropose::default().instantiate(
        deps.branch(),
        env,
        info,
        InstantiateBase {
            deposit_info: None,
            open_proposal_submission: true,
        },
    )?;

    let config = Config {
        main_dao: deps.api.addr_validate(msg.main_dao.as_str())?,
    };

    CONFIG.save(deps.storage, &config)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, PreProposeOverruleError> {
    // We don't want to expose the `proposer` field on the propose
    // message externally as that is to be set by this module. Here,
    // we transform an external message which omits that field into an
    // internal message which sets it.
    type ExecuteInternal = ExecuteBase<ProposeMessageInternal>;
    match msg {
        ExecuteMsg::Propose {
            msg:
                ProposeMessage::ProposeOverrule {
                    timelock_contract,
                    proposal_id,
                },
        } => {
            let timelock_contract_addr = deps.api.addr_validate(&timelock_contract)?;

            let overrule_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: timelock_contract_addr.to_string(),
                msg: to_binary(&TimelockExecuteMsg::OverruleProposal { proposal_id })?,
                funds: vec![],
            });

            let internal_msg = ExecuteInternal::Propose {
                msg: ProposeMessageInternal::Propose {
                    // Fill in proposer based on message sender.
                    proposer: Some(info.sender.to_string()),
                    title: "Overrule proposal".to_string(),
                    description: "Reject the decision made by subdao".to_string(),
                    msgs: vec![overrule_msg],
                },
            };

            let timelock_config_bin = deps
                .querier
                .query_wasm_smart(timelock_contract.to_string(), &TimelockQueryMsg::Config {})?;
            let timelock_config: TimelockConfig = from_binary(&timelock_config_bin).unwrap();

            let config = CONFIG.load(deps.storage)?;

            let query_msg_2 = MainDaoQueryMsg::ListSubDaos {
                start_after: None,
                limit: Some(10),
            };
            let subdao_list_bin = deps
                .querier
                .query_wasm_smart(config.main_dao.to_string(), &query_msg_2)?;
            let subdao_list: Vec<SubDao> = from_binary(&subdao_list_bin).unwrap();

            //todo pagination handling

            if subdao_list
                .into_iter()
                .find(|x1| x1.addr == timelock_config.subdao)
                .is_none()
            {
                return Err(PreProposeOverruleError::MessageUnsupported {});
            };

            let query_msg_3 = TimelockQueryMsg::Proposal { proposal_id };
            let proposal_bin = deps
                .querier
                .query_wasm_smart(timelock_contract.to_string(), &query_msg_3)?;
            let proposal_from_timelock: SingleChoiceProposal = from_binary(&proposal_bin).unwrap();

            if proposal_from_timelock.status != ProposalStatus::Timelocked {
                return Err(PreProposeOverruleError::MessageUnsupported {});
            };

            PrePropose::default()
                .execute(deps, env, info, internal_msg)
                .map_err(|e| PreProposeOverruleError::PreProposeBase(e))
        }
        _ => Err(PreProposeOverruleError::MessageUnsupported {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    PrePropose::default().query(deps, env, msg)
}
