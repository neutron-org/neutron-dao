use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Uint128};
use cw20::Cw20Coin;

use cw_denom::UncheckedDenom;
use cw_multi_test::{next_block, BankSudo, BasicApp, Executor, SudoMsg};
use cw_utils::Duration;
use cwd_interface::{Admin, ModuleInstantiateInfo};
use cwd_pre_propose_single as cppbps;
use neutron_sdk::bindings::msg::NeutronMsg;

use cwd_voting::{
    deposit::{DepositRefundPolicy, UncheckedDepositInfo},
    pre_propose::PreProposeInfo,
    threshold::{PercentageThreshold, Threshold::ThresholdQuorum},
};

use crate::msg::InstantiateMsg;
use crate::testing::contracts::neutron_vault_contract;

use super::{
    contracts::{cw_core_contract, proposal_single_contract, voting_registry_contract},
    CREATOR_ADDR,
};

const DAO_ADDR: &str = "dao";

pub(crate) fn get_pre_propose_info(
    app: &mut BasicApp<NeutronMsg>,
    deposit_info: Option<UncheckedDepositInfo>,
    open_proposal_submission: bool,
) -> PreProposeInfo {
    let pre_propose_contract =
        app.store_code(crate::testing::contracts::pre_propose_single_contract());
    PreProposeInfo::ModuleMayPropose {
        info: ModuleInstantiateInfo {
            code_id: pre_propose_contract,
            msg: to_binary(&cppbps::InstantiateMsg {
                deposit_info,
                open_proposal_submission,
            })
            .unwrap(),
            admin: Some(Admin::CoreModule {}),
            label: "pre_propose_contract".to_string(),
        },
    }
}

pub(crate) fn get_proposal_module_instantiate(app: &mut BasicApp<NeutronMsg>) -> InstantiateMsg {
    InstantiateMsg {
        threshold: ThresholdQuorum {
            quorum: PercentageThreshold::Percent(Decimal::percent(15)),
            threshold: PercentageThreshold::Majority {},
        },
        max_voting_period: Duration::Time(604800), // One week.
        min_voting_period: None,
        allow_revoting: false,
        pre_propose_info: get_pre_propose_info(
            app,
            Some(UncheckedDepositInfo {
                denom: cwd_voting::deposit::DepositToken::Token {
                    denom: UncheckedDenom::Native("ujuno".parse().unwrap()),
                },
                amount: Uint128::new(10_000_000),
                refund_policy: DepositRefundPolicy::OnlyPassed,
            }),
            false,
        ),
        close_proposal_on_execution_failure: true,
    }
}

pub(crate) fn instantiate_with_native_bonded_balances_governance(
    app: &mut BasicApp<NeutronMsg>,
    proposal_module_instantiate: InstantiateMsg,
    initial_balances: Option<Vec<Cw20Coin>>,
) -> Addr {
    let voting_vault_code_id = app.store_code(neutron_vault_contract());

    let vault_intantiate = neutron_vault::msg::InstantiateMsg {
        name: "Test vault".to_string(),
        description: "based neutron vault".to_string(),
        owner: DAO_ADDR.to_string(),
        denom: "ujuno".to_string(),
    };

    let vault_addr = app
        .instantiate_contract(
            voting_vault_code_id,
            Addr::unchecked(CREATOR_ADDR),
            &vault_intantiate,
            &[],
            "neutron vault",
            None,
        )
        .unwrap();

    let proposal_module_code_id = app.store_code(proposal_single_contract());

    let initial_balances = initial_balances.unwrap_or_else(|| {
        vec![Cw20Coin {
            address: CREATOR_ADDR.to_string(),
            amount: Uint128::new(100_000_000),
        }]
    });

    // Collapse balances so that we can test double votes.
    let initial_balances: Vec<Cw20Coin> = {
        let mut already_seen = vec![];
        initial_balances
            .into_iter()
            .filter(|Cw20Coin { address, amount: _ }| {
                if already_seen.contains(address) {
                    false
                } else {
                    already_seen.push(address.clone());
                    true
                }
            })
            .collect()
    };

    let voting_registry_id = app.store_code(voting_registry_contract());
    let core_contract_id = app.store_code(cw_core_contract());

    let instantiate_core = cwd_core::msg::InstantiateMsg {
        name: "DAO DAO".to_string(),
        description: "A DAO that builds DAOs".to_string(),
        dao_uri: None,
        voting_registry_module_instantiate_info: ModuleInstantiateInfo {
            code_id: voting_registry_id,
            msg: to_binary(&neutron_voting_registry::msg::InstantiateMsg {
                owner: DAO_ADDR.to_string(),
                voting_vaults: vec![vault_addr.to_string()],
            })
            .unwrap(),
            admin: None,
            label: "DAO DAO voting module".to_string(),
        },
        proposal_modules_instantiate_info: vec![ModuleInstantiateInfo {
            code_id: proposal_module_code_id,
            label: "DAO DAO governance module.".to_string(),
            admin: Some(Admin::CoreModule {}),
            msg: to_binary(&proposal_module_instantiate).unwrap(),
        }],
        initial_items: None,
    };

    let core_addr = app
        .instantiate_contract(
            core_contract_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate_core,
            &[],
            "DAO DAO",
            None,
        )
        .unwrap();

    for Cw20Coin { address, amount } in initial_balances {
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: address.clone(),
            amount: vec![Coin {
                denom: "ujuno".to_string(),
                amount,
            }],
        }))
        .unwrap();
        app.execute_contract(
            Addr::unchecked(&address),
            vault_addr.clone(),
            &neutron_vault::msg::ExecuteMsg::Bond {},
            &[Coin {
                amount,
                denom: "ujuno".to_string(),
            }],
        )
        .unwrap();
    }

    app.update_block(next_block);

    core_addr
}
