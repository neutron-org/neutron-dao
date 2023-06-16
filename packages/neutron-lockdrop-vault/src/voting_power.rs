use astroport_periphery::lockdrop::{PoolType, QueryMsg as LockdropQueryMsg};
use cosmwasm_std::{Decimal256, Deps, StdResult, Uint128};
use neutron_oracle::voting_power::voting_power_from_lp_tokens;
use serde::Serialize;

pub fn get_voting_power_for_address(
    deps: Deps,
    lockdrop_contract: impl Into<String>,
    usdc_cl_pool_contract: impl Into<String>,
    atom_cl_pool_contract: impl Into<String>,
    pool_type: PoolType,
    address: String,
    height: u64,
) -> StdResult<Decimal256> {
    let pool_contract: String = match pool_type {
        PoolType::ATOM => atom_cl_pool_contract.into(),
        PoolType::USDC => usdc_cl_pool_contract.into(),
    };

    get_voting_power(
        deps,
        lockdrop_contract,
        pool_contract,
        &LockdropQueryMsg::QueryUserLockupTotalAtHeight {
            pool_type,
            user_address: address,
            height,
        },
        height,
    )
}

pub fn get_voting_power_total(
    deps: Deps,
    lp_contract: impl Into<String>,
    oracle_usdc_contract: impl Into<String>,
    oracle_atom_contract: impl Into<String>,
    pool_type: PoolType,
    height: u64,
) -> StdResult<Decimal256> {
    let oracle_contract: String = match pool_type {
        PoolType::ATOM => oracle_atom_contract.into(),
        PoolType::USDC => oracle_usdc_contract.into(),
    };

    get_voting_power(
        deps,
        lp_contract,
        oracle_contract,
        &LockdropQueryMsg::QueryLockupTotalAtHeight { pool_type, height },
        height,
    )
}

pub fn get_voting_power(
    deps: Deps,
    lockdrop_contract: impl Into<String>,
    pool_contract: String,
    msg: &impl Serialize,
    height: u64,
) -> StdResult<Decimal256> {
    let lp_tokens: Option<Uint128> = deps.querier.query_wasm_smart(lockdrop_contract, msg)?;

    let pair_info: astroport::asset::PairInfo = deps.querier.query_wasm_smart(
        &pool_contract,
        &astroport::pair_concentrated::QueryMsg::Pair {},
    )?;

    let lp_total_supply: Uint128 = deps.querier.query_wasm_smart(
        pair_info.liquidity_token,
        &astroport::xastro_token::QueryMsg::TotalSupplyAt { block: height },
    )?;

    voting_power_from_lp_tokens(
        deps,
        lp_tokens.unwrap_or_default(),
        lp_total_supply,
        pool_contract,
        height,
    )
}
