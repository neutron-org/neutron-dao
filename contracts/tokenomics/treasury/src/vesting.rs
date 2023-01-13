use cosmwasm_std::{Decimal, Deps, DepsMut, StdError, StdResult, Uint128};
use neutron_bindings::{
    bindings::query::InterchainQueries, query::total_burned_neutrons::query_total_burned_neutrons,
};

use crate::state::{LAST_BURNED_COINS_AMOUNT, TOTAL_DISTRIBUTED, TOTAL_RESERVED};

/// Function calculates how many coins should be released for the current period
/// based on the current balance and the number of coins burned for the period
/// Implemented vesting function is linear and is defined as: y=x/vesting_denominator
/// In order to optimize the function, we use the following formula: y=x - ((vesting_denominator-1) / vesting_denominator)^<coins for period> * x
pub fn vesting_function(
    current_balance: Uint128,
    burned_coins_for_period: u32,
    vesting_denominator: u128,
) -> StdResult<Uint128> {
    if current_balance.is_zero() || burned_coins_for_period == 0 {
        return Ok(Uint128::zero());
    }

    let current_balance = Decimal::from_atomics(current_balance, 0).map_err(|err| {
        StdError::generic_err(format!("Unable to convert Uint128 to Decimal. {:?}", err))
    })?;

    let multiplier = Decimal::from_ratio(vesting_denominator - 1, vesting_denominator) // vesting_denominator-1 / vesting_denominator
        .checked_pow(burned_coins_for_period)?; // ^<coins for period>

    let coins_left = multiplier.checked_mul(current_balance)?; // (vesting_denominator-1 / vesting_denominator)^<coins for period> * x

    let rounded = current_balance.checked_sub(coins_left)?.ceil();

    Uint128::try_from(rounded.to_string().as_str())
}

pub fn safe_burned_coins_for_period(
    burned_coins: Uint128,
    last_burned_coins: Uint128,
) -> StdResult<u32> {
    let burned_coins_for_period = burned_coins.checked_sub(last_burned_coins)?;

    if burned_coins_for_period > Uint128::from(u32::MAX) {
        return Ok(u32::MAX);
    }

    u32::try_from(burned_coins_for_period.u128()).map_err(|_err| {
        StdError::generic_err("Burned coins amount for period is too big to be converted to u32")
    })
}

pub fn update_distribution_stats(
    deps: DepsMut<InterchainQueries>,
    to_distribute: Uint128,
    to_reserve: Uint128,
    burned_coins: Uint128,
) -> StdResult<()> {
    // update stats
    let total_distributed = TOTAL_DISTRIBUTED.load(deps.storage)?;
    TOTAL_DISTRIBUTED.save(
        deps.storage,
        &(total_distributed.checked_add(to_distribute)?),
    )?;
    let total_reserved = TOTAL_RESERVED.load(deps.storage)?;
    TOTAL_RESERVED.save(deps.storage, &(total_reserved.checked_add(to_reserve)?))?;

    LAST_BURNED_COINS_AMOUNT.save(deps.storage, &burned_coins)?;

    Ok(())
}

pub fn get_burned_coins(deps: Deps<InterchainQueries>, denom: &String) -> StdResult<Uint128> {
    let res =
        query_total_burned_neutrons(deps).map_err(|err| StdError::generic_err(err.to_string()))?;

    if res.coin.denom == *denom {
        return Ok(res.coin.amount);
    }

    Err(StdError::not_found("Burned coins"))
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use cosmwasm_std::coin;

    use crate::testing::mock_querier::mock_dependencies;

    use super::*;

    const DENOM: &str = "denom";

    #[test]
    fn test_safe_burned_coins_for_period() {
        assert_eq!(
            safe_burned_coins_for_period(Uint128::from(100u128), Uint128::zero()).unwrap(),
            100u32
        );

        assert_eq!(
            safe_burned_coins_for_period(Uint128::from(100u128), Uint128::from(50u128)).unwrap(),
            50u32
        );

        assert_eq!(
            safe_burned_coins_for_period(Uint128::from(u32::MAX), Uint128::zero()).unwrap(),
            u32::MAX
        );

        assert_eq!(
            safe_burned_coins_for_period(Uint128::from_str("5294967295").unwrap(), Uint128::zero())
                .unwrap(),
            u32::MAX
        );

        assert_eq!(
            safe_burned_coins_for_period(
                Uint128::from_str("5294967295").unwrap(),
                Uint128::from(u32::MAX)
            )
            .unwrap(),
            1000000000u32
        );

        assert_eq!(
            safe_burned_coins_for_period(
                Uint128::from_str("5294967295").unwrap(),
                Uint128::from(1000u32)
            )
            .unwrap(),
            u32::MAX
        );
    }

    #[test]
    fn test_get_burned_coins_single_coin() {
        let mut deps = mock_dependencies(&[coin(1000000, DENOM)]);

        deps.querier.set_total_burned_neutrons(coin(100, DENOM));

        let burned_tokens = get_burned_coins(deps.as_ref(), &DENOM.to_string()).unwrap();
        assert_eq!(burned_tokens, Uint128::from(100u128));
    }

    #[test]
    fn test_get_burned_coins_not_supported_denom() {
        let mut deps = mock_dependencies(&[coin(1000000, DENOM)]);

        deps.querier
            .set_total_burned_neutrons(coin(100, "custom_denom"));

        let burned_tokens = get_burned_coins(deps.as_ref(), &DENOM.to_string());
        assert_eq!(
            burned_tokens.err(),
            Some(StdError::not_found("Burned coins"))
        );
    }

    #[test]
    fn test_get_burned_coins_with_query_error() {
        let mut deps = mock_dependencies(&[coin(1000000, DENOM)]);

        deps.querier.set_total_burned_neutrons_error(true);

        let burned_tokens = get_burned_coins(deps.as_ref(), &DENOM.to_string());
        assert_eq!(
            burned_tokens.err(),
            Some(StdError::generic_err(
                "Generic error: Querier contract error: Contract error"
            ))
        );

        deps.querier.set_total_burned_neutrons_error(false);
    }

    #[test]
    fn test_vesting_function_return_value() {
        assert_eq!(
            vesting_function(Uint128::new(100), 1, 2u128).unwrap(),
            Uint128::new(50),
        );

        assert_eq!(
            vesting_function(Uint128::new(100), 2, 3u128).unwrap(),
            Uint128::new(56)
        );

        assert_eq!(
            vesting_function(Uint128::new(100), 4, 4u128).unwrap(),
            Uint128::new(69)
        );

        assert_eq!(
            vesting_function(
                Uint128::new(20_000_000),
                4_294_967_295, // u64::MAX
                100_000_000_000u128
            )
            .unwrap(),
            Uint128::new(840808)
        );

        assert_eq!(
            vesting_function(Uint128::new(1000000000), 4000000, 100_000_000_000u128).unwrap(),
            Uint128::new(40000)
        );

        assert_eq!(
            vesting_function(Uint128::new(100000000), 66666666, 100_000_000_000u128).unwrap(),
            Uint128::new(66645)
        );

        assert_eq!(
            vesting_function(Uint128::new(441978163), 10000000, 100_000_000_000u128).unwrap(),
            Uint128::new(44196)
        );

        assert_eq!(
            vesting_function(Uint128::new(441978163), 66758565, 100_000_000_000u128).unwrap(),
            Uint128::new(294960)
        );

        assert_eq!(
            vesting_function(Uint128::new(441978163), 18989885, 100_000_000_000u128).unwrap(),
            Uint128::new(83924)
        );

        assert_eq!(
            vesting_function(
                Uint128::from_str("441978163000").unwrap(),
                441978163,
                100_000_000_000u128
            )
            .unwrap(),
            Uint128::new(1949136417)
        );

        assert_eq!(
            vesting_function(
                Uint128::from_str("20000000000000").unwrap(),
                2_000_000_000,
                100_000_000_000u128
            )
            .unwrap(),
            Uint128::from_str("396026534292").unwrap()
        );
    }

    #[test]
    fn test_vesting_full_consumption_simulation() {
        let current_balance = Uint128::from_str("20000000000000").unwrap();

        let avg_burned_coins_per_block = Uint128::new(5_000_000);
        let total_blocks = Uint128::new(1000);
        let mut total_burned_coins = avg_burned_coins_per_block
            .checked_mul(total_blocks)
            .unwrap();

        let mut total_vested_coins = Uint128::zero();

        while total_burned_coins > Uint128::from(u32::MAX) {
            total_vested_coins += vesting_function(
                current_balance - total_vested_coins,
                u32::MAX,
                100_000_000_000u128,
            )
            .unwrap();

            total_burned_coins -= Uint128::from(u32::MAX);
        }
        let burned_coins_left = u32::try_from(total_burned_coins.u128()).unwrap();

        total_vested_coins += vesting_function(
            current_balance - total_vested_coins,
            burned_coins_left,
            100_000_000_000u128,
        )
        .unwrap();

        assert_eq!(
            total_vested_coins,
            Uint128::from_str("975411511021").unwrap()
        );
    }
}
