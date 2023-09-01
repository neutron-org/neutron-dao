use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Empty};
use cwd_macros::proposal_module_query;

#[proposal_module_query]
#[allow(dead_code)]
#[cw_serde]
#[derive(QueryResponses)]
enum Test {
    #[returns(Empty)]
    Foo,
    #[returns(Empty)]
    Bar(u64),
    #[returns(Empty)]
    Baz { foobar: u64 },
}

#[test]
fn proposal_module_query_derive() {
    let test = Test::Dao {};

    // If this compiles we have won.
    match test {
        Test::Foo | Test::Bar(_) | Test::Baz { .. } | Test::Dao {} => "yay",
    };
}
