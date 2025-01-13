use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128, Order, testing::{mock_dependencies}};
use cw_storage_plus::{SnapshotMap, Strategy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{Validator, VALIDATORS};

// fn store_validator_at_height(deps: DepsMut, addr: Addr, bonded: bool, total_tokens: Uint128, total_shares: Uint128, height: u64) -> StdResult<()> {
//     let validator = Validator {
//         address: addr.clone(),
//         bonded,
//         total_tokens,
//         total_shares,
//     };
//     VALIDATORS.save(deps.storage, &addr, &validator, height)?;
//     VALIDATORS.update()
//     Ok(())
// }

fn delete_validator_at_height(deps: DepsMut, addr: Addr, height: u64) -> StdResult<()> {
    VALIDATORS.remove(deps.storage, &addr, height)?;
    Ok(())
}

fn retrieve_validators_at_height(deps: DepsMut, height: u64) -> StdResult<Vec<(Addr, Validator)>> {
    let mut result = Vec::new();
    for key_result in VALIDATORS.keys(deps.storage, None, None, Order::Ascending) {
        let key = key_result?;
        match VALIDATORS.may_load_at_height(deps.storage, &key, height)? {
            Some(snapshot_value) => {
                result.push((key.clone(), snapshot_value));
            }
            None => {
                continue;
            }
        }
    }
    Ok(result)
}

fn retrieve_removed_keys_at_height(deps: DepsMut, old_height: u64, new_height: u64) -> StdResult<Vec<Addr>> {
    let mut result = Vec::new();

    for key_result in VALIDATORS.keys(deps.storage, None, None, Order::Ascending) {
        let key = key_result?;

        let existed_at_old = VALIDATORS.may_load_at_height(deps.storage, &key, old_height)?.is_some();
        let existed_at_new = VALIDATORS.may_load_at_height(deps.storage, &key, new_height)?.is_some();

        if existed_at_old && !existed_at_new {
            result.push(key.clone());
        }
    }

    Ok(result)
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{testing::mock_dependencies, Decimal256, Uint128, Addr};
    use cosmwasm_std::testing::mock_env;
    use crate::contract::{after_validator_created, after_validator_removed};
    use crate::state::{Delegation, DELEGATIONS};

    #[test]
    fn test_after_validator_created() {
        let mut deps = mock_dependencies();
        let validator_address = "cosmosvaloper1validator".to_string();

        let response = after_validator_created(deps.as_mut(), mock_env(), validator_address.clone()).unwrap();

        let validator = VALIDATORS.load(deps.as_mut().storage, &Addr::unchecked(&validator_address)).unwrap();
        assert!(validator.active);
        assert_eq!(validator.total_tokens, Uint128::zero());
        assert_eq!(validator.total_shares, Uint128::zero());

        assert_eq!(response.attributes[0].value, "validator_created");
        assert_eq!(response.attributes[1].value, validator_address);
    }

    #[test]
    fn test_after_validator_removed() {
        let mut deps = mock_dependencies();
        let validator_address = "cosmosvaloper1validator".to_string();
        let valcons_address = "cosmosvalcons1validator".to_string();

        store_validator_at_height(
            deps.as_mut(),
            Addr::unchecked(&validator_address),
            true,
            Uint128::new(1000),
            Uint128::new(500),
            1,
        )
            .unwrap();

        let response = after_validator_removed(
            deps.as_mut(),
            mock_env(),
            valcons_address.clone(),
            validator_address.clone(),
        )
            .unwrap();

        let validator = VALIDATORS.load(deps.as_mut().storage, &Addr::unchecked(&validator_address)).unwrap();
        assert!(!validator.active);
        assert_eq!(validator.total_tokens, Uint128::zero());
        assert_eq!(validator.total_shares, Uint128::zero());

        assert_eq!(response.attributes[0].value, "validator_disabled");
    }

    #[test]
    fn test_after_validator_bonded() {
        let mut deps = mock_dependencies();
        let validator_address = "cosmosvaloper1validator".to_string();

        store_validator_at_height(
            deps.as_mut(),
            Addr::unchecked(&validator_address),
            false,
            Uint128::zero(),
            Uint128::zero(),
            0,
        )
            .unwrap();

        let response = after_validator_bonded(deps.as_mut(), mock_env(), validator_address.clone()).unwrap();

        let validator = VALIDATORS.load(deps.as_mut().storage, &Addr::unchecked(&validator_address)).unwrap();
        assert!(validator.active);

        assert_eq!(response.attributes[0].value, "validator_bonded");
    }

    #[test]
    fn test_after_validator_begin_unbonding() {
        let mut deps = mock_dependencies();
        let validator_address = "cosmosvaloper1validator".to_string();

        store_validator_at_height(
            deps.as_mut(),
            Addr::unchecked(&validator_address),
            true,
            Uint128::new(1000),
            Uint128::new(500),
            0,
        )
            .unwrap();

        let response = after_validator_begin_unbonding(deps.as_mut(), mock_env(), validator_address.clone()).unwrap();

        let validator = VALIDATORS.load(deps.as_mut().storage, &Addr::unchecked(&validator_address)).unwrap();
        assert!(!validator.active);
        assert_eq!(validator.total_tokens, Uint128::zero());
        assert_eq!(validator.total_shares, Uint128::zero());

        assert_eq!(response.attributes[0].value, "validator_unbonded");
    }

    #[test]
    fn test_after_delegation_modified() {
        let mut deps = mock_dependencies();
        let validator_address = "cosmosvaloper1validator".to_string();
        let delegator_address = "cosmos1delegator".to_string();

        store_validator_at_height(
            deps.as_mut(),
            Addr::unchecked(&validator_address),
            true,
            Uint128::new(1000),
            Uint128::new(500),
            0,
        )
            .unwrap();

        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&Addr::unchecked(&delegator_address), &Addr::unchecked(&validator_address)),
                &Delegation {
                    delegator_address: Addr::unchecked(&delegator_address),
                    validator_address: Addr::unchecked(&validator_address),
                    shares: Uint128::new(300),
                },
                0,
            )
            .unwrap();

        let response = after_delegation_modified(
            deps.as_mut(),
            mock_env(),
            delegator_address.clone(),
            validator_address.clone(),
        )
            .unwrap();

        assert_eq!(response.attributes[0].value, "after_delegation_modified");
    }

    #[test]
    fn test_before_delegation_removed() {
        let mut deps = mock_dependencies();
        let validator_address = "cosmosvaloper1validator".to_string();
        let delegator_address = "cosmos1delegator".to_string();

        store_validator_at_height(
            deps.as_mut(),
            Addr::unchecked(&validator_address),
            true,
            Uint128::new(1000),
            Uint128::new(500),
            0,
        )
            .unwrap();

        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&Addr::unchecked(&delegator_address), &Addr::unchecked(&validator_address)),
                &Delegation {
                    delegator_address: Addr::unchecked(&delegator_address),
                    validator_address: Addr::unchecked(&validator_address),
                    shares: Uint128::new(300),
                },
                0,
            )
            .unwrap();

        let response = before_delegation_removed(
            deps.as_mut(),
            mock_env(),
            delegator_address.clone(),
            validator_address.clone(),
        )
            .unwrap();

        assert_eq!(response.attributes[0].value, "before_delegation_removed");
    }

    fn before_delegation_removed(p0: DepsMut, p1: _, p2: String, p3: String) -> _ {
        todo!()
    }

    #[test]
    fn test_before_validator_slashed() {
        let mut deps = mock_dependencies();
        let validator_address = "cosmosvaloper1validator".to_string();
        let delegator_address = "cosmos1delegator".to_string();

        store_validator_at_height(
            deps.as_mut(),
            Addr::unchecked(&validator_address),
            true,
            Uint128::new(1000),
            Uint128::new(500),
            0,
        )
            .unwrap();

        DELEGATIONS
            .save(
                deps.as_mut().storage,
                (&Addr::unchecked(&delegator_address), &Addr::unchecked(&validator_address)),
                &Delegation {
                    delegator_address: Addr::unchecked(&delegator_address),
                    validator_address: Addr::unchecked(&validator_address),
                    shares: Uint128::new(300),
                },
                0,
            )
            .unwrap();

        let response = before_validator_slashed(
            deps.as_mut(),
            mock_env(),
            validator_address.clone(),
            Decimal256::percent(50),
        )
            .unwrap();

        assert_eq!(response.attributes[0].value, "before_validator_slashed");
    }
}

