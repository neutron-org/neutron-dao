use astroport_periphery::lockdrop::{PoolType, QueryMsg as LockdropQueryMsg};
use cosmwasm_std::{Decimal256, Deps, StdResult, Uint128};
use neutron_oracle::voting_power::voting_power_from_lp_tokens;
use serde::Serialize;

pub fn get_voting_power_for_address(
    deps: Deps,
    lp_contract: impl Into<String>,
    oracle_usdc_contract: impl Into<String>,
    oracle_atom_contract: impl Into<String>,
    pool_type: PoolType,
    address: String,
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
    lp_contract: impl Into<String>,
    oracle_contract: impl Into<String>,
    msg: &impl Serialize,
    height: u64,
) -> StdResult<Decimal256> {
    let lp_tokens: Option<Uint128> = deps.querier.query_wasm_smart(lp_contract, msg)?;

    voting_power_from_lp_tokens(deps, lp_tokens.unwrap_or_default(), oracle_contract, height)
}
