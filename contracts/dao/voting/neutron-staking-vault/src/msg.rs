use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal256, Uint128, Uint64};
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

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        name: String,
        description: String,
        owner: String,
    },
    AddToBlacklist {
        addresses: Vec<String>,
    },
    RemoveFromBlacklist {
        addresses: Vec<String>, // List of addresses to remove from the blacklist
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    AfterValidatorBonded {
        cons_addr: String,
        val_addr: String,
    },

    AfterValidatorRemoved {
        cons_addr: String,
        val_addr: String,
    },

    AfterValidatorCreated {
        val_addr: String,
    },

    AfterValidatorBeginUnbonding {
        cons_addr: String,
        val_addr: String,
    },

    BeforeValidatorModified {
        val_addr: String,
    },

    BeforeDelegationCreated {
        del_addr: String,
        val_addr: String,
    },

    BeforeDelegationSharesModified {
        del_addr: String,
        val_addr: String,
    },

    BeforeDelegationRemoved {
        del_addr: String,
        val_addr: String,
    },

    AfterDelegationModified {
        del_addr: String,
        val_addr: String,
    },

    BeforeValidatorSlashed {
        val_addr: String,
        fraction: Decimal256,
    },

    AfterUnbondingInitiated {
        id: Uint64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    Config {},

    #[returns(Vec<Addr>)]
    ListBlacklistedAddresses {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(bool)]
    IsAddressBlacklisted { address: String },

    #[returns(Uint128)]
    VotingPowerAtHeight {
        address: String,
        height: Option<u64>,
    },

    #[returns(Uint128)]
    TotalPowerAtHeight { height: Option<u64> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
