use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal256, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct InstantiateMsg {
    /// Name contains the vault name which is used to ease the vault's recognition.
    pub name: String,
    /// Description contains information that characterizes the vault.
    pub description: String,
    /// Owner can update all configs including changing the owner. This will generally be a DAO.
    pub owner: String,
    /// Contract to proxy staking updates to.
    pub staking_proxy_info_contract_address: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        name: Option<String>,
        description: Option<String>,
        owner: Option<String>,
        staking_proxy_info_contract_address: Option<String>,
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

    AfterDelegationModified {
        del_addr: String,
        val_addr: String,
    },

    BeforeDelegationRemoved {
        del_addr: String,
        val_addr: String,
    },

    BeforeValidatorSlashed {
        val_addr: String,
        fraction: Decimal256,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the contract's config.
    #[returns(crate::state::Config)]
    Config {},

    /// Gets the staked (bonded) tokens for given `address` at given `height`.
    /// Stake of unbonded validators does not count.
    #[returns(Uint128)]
    StakeAtHeight {
        address: String,
        height: Option<u64>,
    },

    /// Gets the total staked (bonded) tokens for given `height`.
    /// Stake of unbonded validators does not count.
    #[returns(Uint128)]
    TotalStakeAtHeight { height: Option<u64> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

/// Messages to the staking-info-proxy contract.
#[cw_serde]
pub enum ProxyInfoExecute {
    UpdateStake { user: String },
    Slashing {},
}
