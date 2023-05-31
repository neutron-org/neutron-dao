#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdResult, SubMsg, WasmMsg, from_binary};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use neutron_sdk::bindings::msg::NeutronMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cwd_pre_propose_base::{
    error::PreProposeError,
    msg::{ExecuteMsg as ExecuteBase, InstantiateMsg as InstantiateBase},
    state::PreProposeContract,
};
use neutron_subdao_core::msg::ExecuteMsg as CoreExecuteMsg;
use neutron_subdao_pre_propose_single_no_timelock::msg::MigrateMsg;
use neutron_subdao_pre_propose_single_no_timelock::{
    msg::{ExecuteMsg as ExecuteMsgPause, QueryExt, QueryMsg},
    types::ProposeMessage,
};

pub type InstantiateMsg = InstantiateBase;
pub type ExecuteMsg = ExecuteBase<ProposeMessage>;
pub type QueryMsg = QueryBase<Empty>;

pub(crate) const CONTRACT_NAME: &str = "crates.io:cwd-subdao-pre-propose-single";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const TIMELOCK_MODULE_INSTANTIATE_REPLY_ID: u64 = 1;


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

type PrePropose = PreProposeContract<ProposeMessageInternal, Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, PreProposeError> {
    let resp = PrePropose::default().instantiate(deps.branch(), env, info, msg)?;
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
            for msg in msgs {
                match msg {
                    CosmosMsg::Wasm(w) => match w {
                        WasmMsg::Execute { contract_addr: _contract_addr, msg, funds: _funds } => {
                            from_binary::<ExecuteMsgPause>(&msg).unwrap();
                        }
                        _ => {PreProposeError::NotAPauseMsg}
                    },
                    _ =>  {PreProposeError::NotAPauseMsg},
                }
            }

            ExecuteInternal::Propose {
                msg: ProposeMessageInternal::Propose {
                    // Fill in proposer based on message sender.
                    proposer: Some(info.sender.to_string()),
                    title,
                    description,
                    msgs,
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
