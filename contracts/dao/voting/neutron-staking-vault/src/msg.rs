use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal256, Uint128};
use cwd_interface::voting::InfoResponse;
use cwd_interface::voting::{
    BondingStatusResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use cwd_macros::{info_query, voting_query, voting_vault, voting_vault_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    /// Name contains the vault name which is used to ease the vault's recognition.
    pub name: String,
    // Description contains information that characterizes the vault.
    pub description: String,
    // Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: String,
    // Token denom e.g. untrn, or some ibc denom
    pub denom: String,
}

#[voting_vault]
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        name: String,
        description: String,
        owner: String,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    AfterValidatorBonded {
        val_address: String,
    },
    AfterValidatorBeginUnbonding {
        val_address: String,
    },

    BeforeValidatorSlashed {
        val_address: String,
        slashing_fraction: Decimal256,
    },
    AfterDelegationModified {
        delegator_address: String,
        val_address: String,
    },
    BeforeDelegationRemoved {
        delegator_address: String,
        val_address: String,
    },
    AfterValidatorCreated {
        val_address: String,
    },
    AfterValidatorRemoved {
        valcons_address: String,
        val_address: String,
    },
}

#[voting_query]
#[voting_vault_query]
#[info_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
