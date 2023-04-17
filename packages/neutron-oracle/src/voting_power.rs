use astroport::{asset::AssetInfo, oracle::QueryMsg as OracleQueryMsg};
use cosmwasm_std::{Decimal256, Deps, StdResult, Uint128, Uint256, Uint64};

pub fn voting_power_from_lp_tokens(
    deps: Deps,
    lp_tokens: Uint128,
    oracle_contract: impl Into<String>,
    height: u64,
) -> StdResult<Decimal256> {
    Ok(if lp_tokens.is_zero() {
        Decimal256::zero()
    } else {
        deps.querier
            .query_wasm_smart::<Vec<(AssetInfo, Decimal256)>>(
                oracle_contract,
                &OracleQueryMsg::TWAPAtHeight {
                    token: AssetInfo::NativeToken {
                        denom: "untrn".to_string(),
                    },
                    height: Uint64::new(height),
                },
            )?
            .into_iter()
            .map(|x| x.1)
            .sum::<Decimal256>()
            .sqrt()
            .checked_mul(Decimal256::new(Uint256::from(lp_tokens)))?
    })
}
