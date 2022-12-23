use std::marker::PhantomData;

use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    Coin, OwnedDeps,
};

const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let custom_querier = MockQuerier::new(&[(contract_addr, contract_balance)]);

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}
