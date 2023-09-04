use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Empty;
use cwd_interface::voting::{
    InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};
use cwd_macros::{info_query, voting_query};

/// enum for testing. Important that this derives things / has other
/// attributes so we can be sure we aren't messing with other macros
/// with ours.
#[voting_query]
#[info_query]
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
fn voting_query_derive() {
    let _test = Test::VotingPowerAtHeight {
        address: "foo".to_string(),
        height: Some(10),
    };

    let test = Test::TotalPowerAtHeight { height: Some(10) };

    // If this compiles we have won.
    match test {
        Test::Foo
        | Test::Bar(_)
        | Test::Baz { .. }
        | Test::TotalPowerAtHeight { height: _ }
        | Test::VotingPowerAtHeight {
            height: _,
            address: _,
        }
        | Test::Info {} => "yay",
    };
}
