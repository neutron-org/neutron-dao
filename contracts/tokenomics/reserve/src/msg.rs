use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Addr, CosmosMsg, Decimal, Env, StdResult, Uint128, WasmMsg};
use cwd_macros::{pausable, pausable_query};
use exec_control::pause::PauseInfoResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// Address of the Neutron DAO contract
    pub main_dao_address: String,
    /// Denom of the main coin
    pub denom: String,
    /// Distribution rate (0-1) which goes to distribution contract
    pub distribution_rate: Decimal,
    /// Minimum period between distribution calls
    pub min_period: u64,
    /// Address of distribution contract
    pub distribution_contract: String,
    /// Address of treasury contract
    pub treasury_contract: String,
    /// Address of security DAO contract
    pub security_dao_address: String,
    /// Vesting release function denominator
    pub vesting_denominator: u128,
}

#[pausable]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),

    /// Distribute pending funds between Bank and Distribution accounts
    Distribute {},

    /// Update config
    UpdateConfig {
        distribution_rate: Option<Decimal>,
        min_period: Option<u64>,
        distribution_contract: Option<String>,
        treasury_contract: Option<String>,
        security_dao_address: Option<String>,
        vesting_denominator: Option<u128>,
    },

    /// Processes either partial or full xyk->CL migration of contract's liquidity.
    MigrateFromXykToCl {
        slippage_tolerance: Option<Decimal>,
        ntrn_atom_amount: Option<Uint128>,
        ntrn_usdc_amount: Option<Uint128>,
    },

    /// Callbacks; only callable by the contract itself.
    Callback(CallbackMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    MigrateLiquidityToClPair {
        xyk_pair: Addr,
        xyk_lp_token: Addr,
        amount: Uint128,
        slippage_tolerance: Decimal,
        cl_pair: Addr,
        ntrn_denom: String,
        paired_asset_denom: String,
    },
    ProvideLiquidityToClPairAfterWithdrawal {
        ntrn_denom: String,
        ntrn_init_balance: Uint128,
        paired_asset_denom: String,
        paired_asset_init_balance: Uint128,
        cl_pair: Addr,
        slippage_tolerance: Decimal,
    },
    PostMigrationBalancesCheck {
        ntrn_denom: String,
        ntrn_init_balance: Uint128,
        paired_asset_denom: String,
        paired_asset_init_balance: Uint128,
    },
}

// Modified from
// https://github.com/CosmWasm/cosmwasm-plus/blob/v0.2.3/packages/cw20/src/receiver.rs#L15
impl CallbackMsg {
    pub fn to_cosmos_msg(self, env: &Env) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Callback(self))?,
            funds: vec![],
        }))
    }
}

#[pausable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// The contract's configurations; returns [`ConfigResponse`]
    #[returns(crate::state::Config)]
    Config {},
    /// The contract's current stats; returns [`StatsResponse`]    
    #[returns(StatsResponse)]
    Stats {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StatsResponse {
    /// Amount of coins distributed since contract instantiation
    pub total_distributed: Uint128,
    /// Amount of coins reserved since contract instantiation
    pub total_reserved: Uint128,
    /// Total amount of burned coins processed by reserve contract
    pub total_processed_burned_coins: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributeMsg {
    Fund {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {
    pub max_slippage: Decimal,
    pub ntrn_denom: String,
    pub atom_denom: String,
    pub ntrn_atom_xyk_pair: String,
    pub ntrn_atom_cl_pair: String,
    pub usdc_denom: String,
    pub ntrn_usdc_xyk_pair: String,
    pub ntrn_usdc_cl_pair: String,
}
