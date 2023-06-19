use astroport::asset::Decimal256Ext;
use cosmwasm_std::{Decimal256, Deps, StdResult, Uint128, Uint64};
use std::ops::Div;

pub fn voting_power_from_lp_tokens(
    deps: Deps,
    lp_tokens: Uint128,
    total_lp_tokens: Uint128,
    cl_pool: impl Into<String>,
    height: u64,
) -> StdResult<Decimal256> {
    if lp_tokens.is_zero() {
        Ok(Decimal256::zero())
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
            return Ok(Decimal256::zero());
        };

        if ntrn_balance_in_pool.is_zero() {
            return Ok(Decimal256::zero());
        }

        Ok(Decimal256::from_ratio(lp_tokens, total_lp_tokens)
            .div(Decimal256::from_integer(ntrn_balance_in_pool)))
    }
}
