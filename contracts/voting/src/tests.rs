use helpers::{
    assert_voting_power_eq, assert_voting_powers_eq, execute_lock, execute_unlock,
    expect_not_enough_funds_error,
};

const OWNER: &str = "owner";
const DENOM: &str = "denom";
const USER1: &str = "user1";
const USER2: &str = "user2";

#[test]
fn no_voting_power_with_zero_funds_locked() {
    let deps = helpers::init(OWNER, DENOM).unwrap();
    assert_voting_power_eq(deps.as_ref(), USER1, 0);
}

#[test]
fn locked_funds_provide_some_voting_power() {
    let mut deps = helpers::init(OWNER, DENOM).unwrap();
    execute_lock(deps.as_mut(), USER1, 200, DENOM).unwrap();
    assert_voting_power_eq(deps.as_ref(), USER1, 200);
}

#[test]
fn no_voting_power_after_unlocking_all_funds() {
    let mut deps = helpers::init(OWNER, DENOM).unwrap();
    execute_lock(deps.as_mut(), USER1, 200, DENOM).unwrap();
    execute_unlock(deps.as_mut(), USER1, 200, DENOM).unwrap();
    assert_voting_power_eq(deps.as_ref(), USER1, 0);
}

#[test]
fn voting_power_updates_in_real_time() {
    let mut deps = helpers::init(OWNER, DENOM).unwrap();
    execute_lock(deps.as_mut(), USER1, 120, DENOM).unwrap();
    assert_voting_power_eq(deps.as_ref(), USER1, 120);
    execute_lock(deps.as_mut(), USER1, 60, DENOM).unwrap();
    assert_voting_power_eq(deps.as_ref(), USER1, 180);
    execute_unlock(deps.as_mut(), USER1, 80, DENOM).unwrap();
    assert_voting_power_eq(deps.as_ref(), USER1, 100);
    execute_unlock(deps.as_mut(), USER1, 100, DENOM).unwrap();
    assert_voting_power_eq(deps.as_ref(), USER1, 0);
}

#[test]
fn voting_powers_two_users() {
    let mut deps = helpers::init(OWNER, DENOM).unwrap();
    execute_lock(deps.as_mut(), USER1, 110, DENOM).unwrap();
    execute_lock(deps.as_mut(), USER2, 130, DENOM).unwrap();
    assert_voting_power_eq(deps.as_ref(), USER1, 110);
    assert_voting_power_eq(deps.as_ref(), USER2, 130);
    assert_voting_powers_eq(deps.as_ref(), vec![(USER1, 110), (USER2, 130)])
}

#[test]
fn cannot_unlock_more_than_locked() {
    let mut deps = helpers::init(OWNER, DENOM).unwrap();
    execute_lock(deps.as_mut(), USER1, 110, DENOM).unwrap();
    expect_not_enough_funds_error(execute_unlock(deps.as_mut(), USER1, 120, DENOM), 120, 110);
}

mod helpers {
    use crate::{
        contract::{execute, instantiate, query},
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VotingPowerResponse},
        types::{ContractError, ContractResult},
    };
    use cosmwasm_std::{
        attr, coin, from_binary,
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
        BankMsg, CosmosMsg, Deps, DepsMut, OwnedDeps, Response, Uint128,
    };

    pub fn init(
        owner: &str,
        denom: &str,
    ) -> ContractResult<OwnedDeps<MockStorage, MockApi, MockQuerier>> {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(owner, &[]),
            InstantiateMsg {
                owner: owner.to_string(),
                denom: denom.to_string(),
            },
        )?;
        Ok(deps)
    }

    pub fn execute_lock(
        deps: DepsMut,
        user: &str,
        amount: u128,
        denom: &str,
    ) -> ContractResult<Response> {
        let response = execute(
            deps,
            mock_env(),
            mock_info(user, &[coin(amount, denom)]),
            ExecuteMsg::LockFunds {},
        )?;

        // if lock has not failed, validate emitted attributes
        assert_eq!(response.messages.len(), 0);
        assert_eq!(
            response.attributes,
            vec![
                attr("action", "lock_funds"),
                attr("user", user),
                attr("amount", format!("{}{}", amount, denom))
            ]
        );

        Ok(response)
    }

    pub fn execute_unlock(
        deps: DepsMut,
        user: &str,
        amount: u128,
        denom: &str,
    ) -> ContractResult<Response> {
        let response = execute(
            deps,
            mock_env(),
            mock_info(user, &[]),
            ExecuteMsg::UnlockFunds {
                amount: Uint128::new(amount),
            },
        )?;

        // if unlock has not failed, validate emitted message & attributes
        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: user.into(),
                amount: vec![coin(amount, denom)]
            })
        );
        assert_eq!(
            response.attributes,
            vec![
                attr("action", "unlock_funds"),
                attr("user", user),
                attr("amount", format!("{}{}", amount, denom))
            ]
        );

        Ok(response)
    }

    pub fn query_voting_power(deps: Deps, user: &str) -> ContractResult<VotingPowerResponse> {
        Ok(from_binary(&query(
            deps,
            mock_env(),
            QueryMsg::VotingPower { user: user.into() },
        )?)?)
    }

    pub fn query_voting_powers(deps: Deps) -> ContractResult<Vec<VotingPowerResponse>> {
        Ok(from_binary(&query(
            deps,
            mock_env(),
            QueryMsg::VotingPowers {},
        )?)?)
    }

    pub fn assert_voting_power_eq(deps: Deps, user: &str, expected_voting_power: u128) {
        assert_eq!(
            query_voting_power(deps, user).unwrap(),
            VotingPowerResponse {
                user: user.to_string(),
                voting_power: Uint128::new(expected_voting_power),
            }
        );
    }

    pub fn assert_voting_powers_eq(deps: Deps, expected_voting_powers: Vec<(&str, u128)>) {
        let mut expected = expected_voting_powers
            .into_iter()
            .map(|(user, amount)| VotingPowerResponse {
                user: user.into(),
                voting_power: Uint128::new(amount),
            })
            .collect::<Vec<_>>();
        let mut actual = query_voting_powers(deps).unwrap();

        expected.sort_by(|a, b| a.user.cmp(&b.user));
        actual.sort_by(|a, b| a.user.cmp(&b.user));
        assert_eq!(expected, actual);
    }

    pub fn expect_not_enough_funds_error(
        response: ContractResult<Response>,
        requested: u128,
        has: u128,
    ) {
        assert_eq!(
            response.unwrap_err(),
            ContractError::NotEnoughFundsToUnlock {
                requested: requested.into(),
                has: has.into(),
            }
        )
    }
}
