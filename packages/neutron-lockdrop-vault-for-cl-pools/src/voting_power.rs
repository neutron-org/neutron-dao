use astroport_periphery::lockdrop::{PoolType, QueryMsg as LockdropQueryMsg};
use cosmwasm_std::{Addr, Deps, StdResult, Uint128};
use neutron_voting_power::voting_power::voting_power_from_lp_tokens;
use serde::Serialize;

pub fn get_voting_power_for_address(
    deps: Deps,
    lockdrop_contract: &Addr,
    pool_contract: &Addr,
    pool_type: PoolType,
    address: String,
    height: u64,
) -> StdResult<Uint128> {
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
    lp_contract: &Addr,
    pool_contract: &Addr,
    pool_type: PoolType,
    height: u64,
) -> StdResult<Uint128> {
    get_voting_power(
        deps,
        lp_contract,
        pool_contract,
        &LockdropQueryMsg::QueryLockupTotalAtHeight { pool_type, height },
        height,
    )
}

pub fn get_voting_power(
    deps: Deps,
    lockdrop_contract: &Addr,
    pool_contract: &Addr,
    msg: &impl Serialize,
    height: u64,
) -> StdResult<Uint128> {
    let lp_tokens: Option<Uint128> = deps.querier.query_wasm_smart(lockdrop_contract, msg)?;

    let pair_info: astroport::asset::PairInfo = deps.querier.query_wasm_smart(
        pool_contract,
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
