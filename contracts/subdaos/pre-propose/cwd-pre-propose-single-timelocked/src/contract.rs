#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};
use cw2::set_contract_version;
use neutron_bindings::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::TIMELOCK_CONTRACT;
use cwd_pre_propose_base::{
    error::PreProposeError,
    msg::{ExecuteMsg as ExecuteBase, InstantiateMsg as InstantiateBase, QueryMsg as QueryBase},
    state::PreProposeContract,
};
use cwd_voting::deposit::UncheckedDepositInfo;
use neutron_timelock::single::{ExecuteMsg as TimelockExecuteMsg, ProposalQueryMsg};

pub(crate) const CONTRACT_NAME: &str = "crates.io:cwd-pre-propose-single-timelocked";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProposeMessage {
    Propose {
        title: String,
        description: String,
        msgs: Vec<CosmosMsg<NeutronMsg>>,
    },
}

pub type ExecuteMsg = ExecuteBase<ProposeMessage>;
pub type QueryMsg = QueryBase;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Information about the deposit requirements for this
    /// module. None if no deposit.
    pub deposit_info: Option<UncheckedDepositInfo>,
    /// If false, only members (addresses with voting power) may create
    /// proposals in the DAO. Otherwise, any address may create a
    /// proposal so long as they pay the deposit.
    pub open_proposal_submission: bool,

    /// Address of the timelock contract associated with this pre-propose
    /// module.
    pub timelock_contract: Addr,
}

/// Internal version of the propose message that includes the
/// `proposer` field. The module will fill this in based on the sender
/// of the external message.
#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum ProposeMessageInternal {
    Propose {
        title: String,
        description: String,
        msgs: Vec<CosmosMsg<NeutronMsg>>,
        proposer: Option<String>,
    },
}

type PrePropose = PreProposeContract<ProposeMessageInternal>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, PreProposeError> {
    // Save the timelock contract address. This will be used in the ExecuteMsg::Propose
    // handler to wrap the initial messages with a TimelockProposal message.
    TIMELOCK_CONTRACT.save(deps.storage, &msg.timelock_contract)?;

    let resp = PrePropose::default().instantiate(
        deps.branch(),
        env,
        info,
        InstantiateBase {
            deposit_info: msg.deposit_info,
            open_proposal_submission: msg.open_proposal_submission,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, PreProposeError> {
    // We don't want to expose the `proposer` field on the propose
    // message externally as that is to be set by this module. Here,
    // we transform an external message which omits that field into an
    // internal message which sets it.
    type ExecuteInternal = ExecuteBase<ProposeMessageInternal>;
    let internalized = match msg {
        ExecuteMsg::Propose {
            msg:
                ProposeMessage::Propose {
                    title,
                    description,
                    msgs,
                },
        } => {
            let pre_propose = PrePropose::default();

            let proposal_module = pre_propose.proposal_module.load(deps.storage)?;
            let timelock_contract = TIMELOCK_CONTRACT.load(deps.storage)?;

            let last_proposal_id: u64 = deps.querier.query_wasm_smart(
                proposal_module.to_string(),
                &ProposalQueryMsg::ProposalCount {},
            )?;

            let timelock_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: timelock_contract.to_string(),
                msg: to_binary(&TimelockExecuteMsg::TimelockProposal {
                    proposal_id: last_proposal_id + 1,
                    msgs,
                })
                .unwrap(),
                funds: vec![],
            });

            ExecuteInternal::Propose {
                msg: ProposeMessageInternal::Propose {
                    // Fill in proposer based on message sender.
                    proposer: Some(info.sender.to_string()),
                    title,
                    description,
                    msgs: vec![timelock_msg],
                },
            }
        }
        ExecuteMsg::Withdraw { denom } => ExecuteInternal::Withdraw { denom },
        ExecuteMsg::UpdateConfig {
            deposit_info,
            open_proposal_submission,
        } => ExecuteInternal::UpdateConfig {
            deposit_info,
            open_proposal_submission,
        },
        ExecuteMsg::ProposalCreatedHook {
            proposal_id,
            proposer,
        } => ExecuteInternal::ProposalCreatedHook {
            proposal_id,
            proposer,
        },
        ExecuteMsg::ProposalCompletedHook {
            proposal_id,
            new_status,
        } => ExecuteInternal::ProposalCompletedHook {
            proposal_id,
            new_status,
        },
    };

    PrePropose::default().execute(deps, env, info, internalized)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    PrePropose::default().query(deps, env, msg)
}
