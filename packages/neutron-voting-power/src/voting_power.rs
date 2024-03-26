use cosmwasm_std::{Addr, Deps, StdResult, Uint128, Uint64};

pub fn voting_power_from_lp_tokens(
    deps: Deps,
    lp_tokens: Uint128,
    total_lp_tokens: Uint128,
    cl_pool: &Addr,
    height: u64,
) -> StdResult<Uint128> {
    if lp_tokens.is_zero() {
        Ok(Uint128::zero())
    } else {
        let balance_resp: Option<Uint128> = deps.querier.query_wasm_smart(
            cl_pool,
            &astroport::pair_concentrated::QueryMsg::AssetBalanceAt {
                asset_info: astroport::asset::AssetInfo::NativeToken {
                    denom: "untrn".to_string(),
                },
                block_height: Uint64::from(height),
            },
        )?;
        let ntrn_balance_in_pool = if let Some(ntrn_balance) = balance_resp {
            ntrn_balance
        } else {
            return Ok(Uint128::zero());
        };

        if ntrn_balance_in_pool.is_zero() {
            return Ok(Uint128::zero());
        }

        Ok(lp_tokens.multiply_ratio(ntrn_balance_in_pool, total_lp_tokens))
    }
}
