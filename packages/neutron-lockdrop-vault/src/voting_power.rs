use astroport::{asset::AssetInfo, oracle::QueryMsg as OracleQueryMsg};
use astroport_periphery::lockdrop::{PoolType, QueryMsg as LockdropQueryMsg};
use cosmwasm_std::{Decimal256, Deps, StdResult, Uint128, Uint256, Uint64};
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

    let lp_tokens = Decimal256::new(Uint256::from(lp_tokens.unwrap_or_default()));

    if lp_tokens.is_zero() {
        return Ok(Decimal256::zero());
    }

    let twap: Vec<(AssetInfo, Decimal256)> = deps.querier.query_wasm_smart(
        oracle_contract,
        &OracleQueryMsg::TWAPAtHeight {
            token: AssetInfo::NativeToken {
                denom: "untrn".to_string(),
            },
            height: Uint64::new(height),
        },
    )?;

    let untrn_twap_assets = twap.iter().map(|x| x.1);

    let total_twap_untrn = untrn_twap_assets.sum::<Decimal256>().sqrt();

    Ok(total_twap_untrn.checked_mul(lp_tokens)?)
}
