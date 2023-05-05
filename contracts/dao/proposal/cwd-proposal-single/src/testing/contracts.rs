use cw_multi_test::{Contract, ContractWrapper};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_vault as vault;

pub(crate) fn neutron_vault_contract() -> Box<dyn Contract<NeutronMsg>> {
    let contract: ContractWrapper<_, _, _, _, _, _, NeutronMsg> = ContractWrapper::new_with_empty(
        vault::contract::execute,
        vault::contract::instantiate,
        vault::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn proposal_single_contract() -> Box<dyn Contract<NeutronMsg>> {
    let contract: ContractWrapper<_, _, _, _, _, _, NeutronMsg, _, _, _, _, _, _> =
        ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply_empty(crate::contract::reply);
    Box::new(contract)
}

pub(crate) fn pre_propose_single_contract() -> Box<dyn Contract<NeutronMsg>> {
    let contract: ContractWrapper<_, _, _, _, _, _, NeutronMsg> = ContractWrapper::new_with_empty(
        cwd_pre_propose_single::contract::execute,
        cwd_pre_propose_single::contract::instantiate,
        cwd_pre_propose_single::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn cw_core_contract() -> Box<dyn Contract<NeutronMsg>> {
    let contract = ContractWrapper::new(
        cwd_core::contract::execute,
        cwd_core::contract::instantiate,
        cwd_core::contract::query,
    )
    .with_reply(cwd_core::contract::reply);
    Box::new(contract)
}

pub(crate) fn voting_registry_contract() -> Box<dyn Contract<NeutronMsg>> {
    let contract: ContractWrapper<_, _, _, _, _, _, NeutronMsg, _, _, _, _, _, _> =
        ContractWrapper::new_with_empty(
            neutron_voting_registry::contract::execute,
            neutron_voting_registry::contract::instantiate,
            neutron_voting_registry::contract::query,
        )
        .with_reply_empty(crate::contract::reply);
    Box::new(contract)
}
