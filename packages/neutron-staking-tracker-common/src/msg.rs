use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal256, Uint128};

#[cw_serde]
#[serde(rename_all = "snake_case")]
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

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        name: Option<String>,
        description: Option<String>,
        owner: Option<String>,
        staking_proxy_info_contract_address: Option<String>,
    },
}

#[cw_serde]
pub enum SudoMsg {
    AfterValidatorCreated {
        val_addr: String,
    },

    AfterValidatorRemoved {
        cons_addr: String,
        val_addr: String,
    },

    AfterValidatorBonded {
        cons_addr: String,
        val_addr: String,
    },

    AfterValidatorBeginUnbonding {
        cons_addr: String,
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
        tokens_to_burn: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the contract's config.
    #[returns(crate::types::Config)]
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

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
