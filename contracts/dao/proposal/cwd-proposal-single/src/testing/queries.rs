use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::BasicApp;
use cwd_core::state::{ProposalModule, ProposalModuleStatus};
use neutron_sdk::bindings::msg::NeutronMsg;

use cwd_hooks::HooksResponse;
use cwd_pre_propose_single as cppbps;
use cwd_voting::pre_propose::ProposalCreationPolicy;

use crate::{
    msg::QueryMsg,
    query::{ProposalListResponse, ProposalResponse, VoteListResponse},
    state::Config,
};

pub(crate) fn query_deposit_config_and_pre_propose_module(
    app: &BasicApp<NeutronMsg>,
    proposal_single: &Addr,
) -> (cppbps::Config, Addr) {
    let proposal_creation_policy = query_creation_policy(app, proposal_single);

    if let ProposalCreationPolicy::Module { addr: module_addr } = proposal_creation_policy {
        let deposit_config = query_pre_proposal_single_config(app, &module_addr);

        (deposit_config, module_addr)
    } else {
        panic!("no pre-propose module.")
    }
}

pub(crate) fn query_proposal_config(app: &BasicApp<NeutronMsg>, proposal_single: &Addr) -> Config {
    app.wrap()
        .query_wasm_smart(proposal_single, &QueryMsg::Config {})
        .unwrap()
}

pub(crate) fn query_creation_policy(
    app: &BasicApp<NeutronMsg>,
    proposal_single: &Addr,
) -> ProposalCreationPolicy {
    app.wrap()
        .query_wasm_smart(proposal_single, &QueryMsg::ProposalCreationPolicy {})
        .unwrap()
}

pub(crate) fn query_list_proposals(
    app: &BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    start_after: Option<u64>,
    limit: Option<u64>,
) -> ProposalListResponse {
    app.wrap()
        .query_wasm_smart(
            proposal_single,
            &QueryMsg::ListProposals { start_after, limit },
        )
        .unwrap()
}

pub(crate) fn query_list_votes(
    app: &BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    proposal_id: u64,
    start_after: Option<String>,
    limit: Option<u64>,
) -> VoteListResponse {
    app.wrap()
        .query_wasm_smart(
            proposal_single,
            &QueryMsg::ListVotes {
                proposal_id,
                start_after,
                limit,
            },
        )
        .unwrap()
}

pub(crate) fn query_proposal_hooks(
    app: &BasicApp<NeutronMsg>,
    proposal_single: &Addr,
) -> HooksResponse {
    app.wrap()
        .query_wasm_smart(proposal_single, &QueryMsg::ProposalHooks {})
        .unwrap()
}

pub(crate) fn query_vote_hooks(
    app: &BasicApp<NeutronMsg>,
    proposal_single: &Addr,
) -> HooksResponse {
    app.wrap()
        .query_wasm_smart(proposal_single, &QueryMsg::VoteHooks {})
        .unwrap()
}

pub(crate) fn query_list_proposals_reverse(
    app: &BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    start_before: Option<u64>,
    limit: Option<u64>,
) -> ProposalListResponse {
    app.wrap()
        .query_wasm_smart(
            proposal_single,
            &QueryMsg::ReverseProposals {
                start_before,
                limit,
            },
        )
        .unwrap()
}

pub(crate) fn query_pre_proposal_single_config(
    app: &BasicApp<NeutronMsg>,
    pre_propose: &Addr,
) -> cppbps::Config {
    app.wrap()
        .query_wasm_smart(pre_propose, &cppbps::QueryMsg::Config {})
        .unwrap()
}

pub(crate) fn query_single_proposal_module(app: &BasicApp<NeutronMsg>, core_addr: &Addr) -> Addr {
    let modules: Vec<ProposalModule> = app
        .wrap()
        .query_wasm_smart(
            core_addr,
            &cwd_core::msg::QueryMsg::ProposalModules {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    // Filter out disabled modules.
    let modules = modules
        .into_iter()
        .filter(|module| module.status == ProposalModuleStatus::Enabled)
        .collect::<Vec<_>>();

    assert_eq!(
        modules.len(),
        1,
        "wrong proposal module count. expected 1, got {}",
        modules.len()
    );

    modules.into_iter().next().unwrap().address
}

pub(crate) fn query_balance_native(app: &BasicApp<NeutronMsg>, who: &str, denom: &str) -> Uint128 {
    let res = app.wrap().query_balance(who, denom).unwrap();
    res.amount
}

pub(crate) fn query_proposal(
    app: &BasicApp<NeutronMsg>,
    proposal_single: &Addr,
    id: u64,
) -> ProposalResponse {
    app.wrap()
        .query_wasm_smart(proposal_single, &QueryMsg::Proposal { proposal_id: id })
        .unwrap()
}
