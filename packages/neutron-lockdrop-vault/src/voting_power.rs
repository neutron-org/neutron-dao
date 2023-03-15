use astroport::{asset::AssetInfo, oracle::QueryMsg as OracleQueryMsg};
use astroport_periphery::lockdrop::{PoolType, QueryMsg as LockdropQueryMsg};
use cosmwasm_std::{Decimal256, Deps, StdResult, Uint128, Uint256, Uint64};

pub fn get_voting_power_for_address(
    deps: Deps,
    lockdrop_contract: impl Into<String>,
    oracle_contract: impl Into<String>,
    pool_type: PoolType,
    address: String,
    height: u64,
) -> StdResult<Decimal256> {
    get_voting_power(
        deps,
        lockdrop_contract,
        oracle_contract,
        pool_type,
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
    lockdrop_contract: impl Into<String>,
    oracle_contract: impl Into<String>,
    pool_type: PoolType,
    height: u64,
) -> StdResult<Decimal256> {
    get_voting_power(
        deps,
        lockdrop_contract,
        oracle_contract,
        pool_type,
        &LockdropQueryMsg::QueryLockupTotalAtHeight { pool_type, height },
        height,
    )
}

pub fn get_voting_power(
    deps: Deps,
    lockdrop_contract: impl Into<String>,
    oracle_contract: impl Into<String>,
    pool_type: PoolType,
    msg: &LockdropQueryMsg,
    height: u64,
) -> StdResult<Decimal256> {
    let lp_tokens: Option<Uint128> = deps.querier.query_wasm_smart(lockdrop_contract, msg)?;

    let lp_tokens = Decimal256::new(Uint256::from(lp_tokens.unwrap_or_default()));

    if lp_tokens.is_zero() {
        return Ok(Decimal256::zero());
    }

    let asset_denom = match pool_type {
        PoolType::ATOM => "uatom",
        PoolType::USDC => "usdc",
    };

    let twap: Vec<(AssetInfo, Decimal256)> = deps.querier.query_wasm_smart(
        oracle_contract,
        &OracleQueryMsg::TWAPAtHeight {
            token: AssetInfo::NativeToken {
                denom: asset_denom.to_string(),
            },
            height: Uint64::new(height),
        },
    )?;

    let untrn_twap_assets = twap
        .iter()
        .filter(|x| {
            let (asset, _amount) = x;

            match asset {
                AssetInfo::NativeToken { denom } => denom == asset_denom,
                _ => false,
            }
        })
        .map(|x| x.1);

    let total_twap_untrn = untrn_twap_assets.sum::<Decimal256>().sqrt();

    Ok(total_twap_untrn.checked_mul(lp_tokens)?)
}
