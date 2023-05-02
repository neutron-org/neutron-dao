use cosmwasm_std::{coins, Addr, Coin, CosmosMsg};
use cw_multi_test::{BankSudo, BasicApp, Executor};
use neutron_sdk::bindings::msg::NeutronMsg;

use cw_denom::CheckedDenom;
use cwd_pre_propose_single as cppbps;
use cwd_voting::{deposit::CheckedDepositInfo, pre_propose::ProposalCreationPolicy, voting::Vote};

use crate::{
    msg::{ExecuteMsg, QueryMsg},
    query::ProposalResponse,
    testing::queries::query_creation_policy,
    ContractError,
};

use super::queries::query_pre_proposal_single_config;

// Creates a proposal then checks that the proposal was created with
// the specified messages and returns the ID of the proposal.
//
// This expects that the proposer already has the needed tokens to pay
// the deposit.
pub(crate) fn make_proposal(
    app: &mut BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    proposer: &str,
    msgs: Vec<CosmosMsg<NeutronMsg>>,
) -> u64 {
    let proposal_creation_policy = query_creation_policy(app, proposal_single);

    // Collect the funding.
    let funds = match proposal_creation_policy {
        ProposalCreationPolicy::Anyone {} => vec![],
        ProposalCreationPolicy::Module {
            addr: ref pre_propose,
        } => {
            let deposit_config = query_pre_proposal_single_config(app, pre_propose);
            match deposit_config.deposit_info {
                Some(CheckedDepositInfo {
                    denom,
                    amount,
                    refund_policy: _,
                }) => match denom {
                    CheckedDenom::Native(denom) => coins(amount.u128(), denom),
                    CheckedDenom::Cw20(addr) => {
                        // Give an allowance, no funds.
                        app.execute_contract(
                            Addr::unchecked(proposer),
                            addr,
                            &cw20::Cw20ExecuteMsg::IncreaseAllowance {
                                spender: pre_propose.to_string(),
                                amount,
                                expires: None,
                            },
                            &[],
                        )
                        .unwrap();
                        vec![]
                    }
                },
                None => vec![],
            }
        }
    };

    // Make the proposal.
    let res = match proposal_creation_policy {
        ProposalCreationPolicy::Anyone {} => app
            .execute_contract(
                Addr::unchecked(proposer),
                proposal_single.clone(),
                &ExecuteMsg::Propose {
                    title: "title".to_string(),
                    description: "description".to_string(),
                    msgs: msgs.clone(),
                    proposer: None,
                },
                &[],
            )
            .unwrap(),
        ProposalCreationPolicy::Module { addr } => app
            .execute_contract(
                Addr::unchecked(proposer),
                addr,
                &cppbps::ExecuteMsg::Propose {
                    msg: cppbps::ProposeMessage::Propose {
                        title: "title".to_string(),
                        description: "description".to_string(),
                        msgs: msgs.clone(),
                    },
                },
                &funds,
            )
            .unwrap(),
    };

    // The new proposal hook is the last message that fires in
    // this process so we get the proposal ID from it's
    // attributes. We could do this by looking at the proposal
    // creation attributes but this changes relative position
    // depending on if a cw20 or native deposit is being used.
    let attrs = res.custom_attrs(res.events.len() - 1);
    let id = attrs[attrs.len() - 1]
        .value
        .parse()
        // If the proposal creation policy doesn't involve a
        // pre-propose module, no hook so we do it manaually.
        .unwrap_or_else(|_| res.custom_attrs(1)[2].value.parse().unwrap());

    // Check that the proposal was created as expected.
    let proposal: ProposalResponse = app
        .wrap()
        .query_wasm_smart(proposal_single, &QueryMsg::Proposal { proposal_id: id })
        .unwrap();

    assert_eq!(proposal.proposal.proposer, Addr::unchecked(proposer));
    assert_eq!(proposal.proposal.title, "title".to_string());
    assert_eq!(proposal.proposal.description, "description".to_string());
    assert_eq!(proposal.proposal.msgs, msgs);

    id
}

pub(crate) fn vote_on_proposal(
    app: &mut BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    sender: &str,
    proposal_id: u64,
    vote: Vote,
) {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_single.clone(),
        &ExecuteMsg::Vote { proposal_id, vote },
        &[],
    )
    .unwrap();
}

pub(crate) fn vote_on_proposal_should_fail(
    app: &mut BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    sender: &str,
    proposal_id: u64,
    vote: Vote,
) -> ContractError {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_single.clone(),
        &ExecuteMsg::Vote { proposal_id, vote },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}

pub(crate) fn execute_proposal_should_fail(
    app: &mut BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    sender: &str,
    proposal_id: u64,
) -> ContractError {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_single.clone(),
        &ExecuteMsg::Execute { proposal_id },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}

pub(crate) fn execute_proposal(
    app: &mut BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    sender: &str,
    proposal_id: u64,
) {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_single.clone(),
        &ExecuteMsg::Execute { proposal_id },
        &[],
    )
    .unwrap();
}

pub(crate) fn close_proposal_should_fail(
    app: &mut BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    sender: &str,
    proposal_id: u64,
) -> ContractError {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_single.clone(),
        &ExecuteMsg::Close { proposal_id },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}

pub(crate) fn close_proposal(
    app: &mut BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    sender: &str,
    proposal_id: u64,
) {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_single.clone(),
        &ExecuteMsg::Close { proposal_id },
        &[],
    )
    .unwrap();
}

pub(crate) fn mint_natives(app: &mut BasicApp<NeutronMsg>, receiver: &str, amount: Vec<Coin>) {
    app.sudo(cw_multi_test::SudoMsg::Bank(BankSudo::Mint {
        to_address: receiver.to_string(),
        amount,
    }))
    .unwrap();
}

pub(crate) fn add_proposal_hook(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module: &Addr,
    sender: &str,
    hook_addr: &str,
) {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_module.clone(),
        &ExecuteMsg::AddProposalHook {
            address: hook_addr.to_string(),
        },
        &[],
    )
    .unwrap();
}

pub(crate) fn add_proposal_hook_should_fail(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module: &Addr,
    sender: &str,
    hook_addr: &str,
) -> ContractError {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_module.clone(),
        &ExecuteMsg::AddProposalHook {
            address: hook_addr.to_string(),
        },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}

pub(crate) fn remove_proposal_hook(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module: &Addr,
    sender: &str,
    hook_addr: &str,
) {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_module.clone(),
        &ExecuteMsg::RemoveProposalHook {
            address: hook_addr.to_string(),
        },
        &[],
    )
    .unwrap();
}

pub(crate) fn remove_proposal_hook_should_fail(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module: &Addr,
    sender: &str,
    hook_addr: &str,
) -> ContractError {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_module.clone(),
        &ExecuteMsg::RemoveProposalHook {
            address: hook_addr.to_string(),
        },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}

pub(crate) fn add_vote_hook(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module: &Addr,
    sender: &str,
    hook_addr: &str,
) {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_module.clone(),
        &ExecuteMsg::AddVoteHook {
            address: hook_addr.to_string(),
        },
        &[],
    )
    .unwrap();
}

pub(crate) fn add_vote_hook_should_fail(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module: &Addr,
    sender: &str,
    hook_addr: &str,
) -> ContractError {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_module.clone(),
        &ExecuteMsg::AddVoteHook {
            address: hook_addr.to_string(),
        },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}

pub(crate) fn remove_vote_hook(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module: &Addr,
    sender: &str,
    hook_addr: &str,
) {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_module.clone(),
        &ExecuteMsg::RemoveVoteHook {
            address: hook_addr.to_string(),
        },
        &[],
    )
    .unwrap();
}

pub(crate) fn remove_vote_hook_should_fail(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module: &Addr,
    sender: &str,
    hook_addr: &str,
) -> ContractError {
    app.execute_contract(
        Addr::unchecked(sender),
        proposal_module.clone(),
        &ExecuteMsg::RemoveVoteHook {
            address: hook_addr.to_string(),
        },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}
