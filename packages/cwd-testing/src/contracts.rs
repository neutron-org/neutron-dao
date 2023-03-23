use cosmwasm_std::Empty;

use cw_multi_test::{Contract, ContractWrapper};
use cwd_pre_propose_multiple as cppm;
use cwd_pre_propose_single as cpps;

pub fn cw4_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
    );
    Box::new(contract)
}

pub fn v1_proposal_single_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cwd_proposal_single_v1::contract::execute,
        cwd_proposal_single_v1::contract::instantiate,
        cwd_proposal_single_v1::contract::query,
    )
    .with_reply(cwd_proposal_single_v1::contract::reply)
    .with_migrate(cwd_proposal_single_v1::contract::migrate);
    Box::new(contract)
}

pub fn proposal_single_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cwd_proposal_single::contract::execute,
        cwd_proposal_single::contract::instantiate,
        cwd_proposal_single::contract::query,
    )
    .with_reply(cwd_proposal_single::contract::reply)
    .with_migrate(cwd_proposal_single::contract::migrate);
    Box::new(contract)
}

pub fn pre_propose_single_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cpps::contract::execute,
        cpps::contract::instantiate,
        cpps::contract::query,
    );
    Box::new(contract)
}

pub fn pre_propose_multiple_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cppm::contract::execute,
        cppm::contract::instantiate,
        cppm::contract::query,
    );
    Box::new(contract)
}
