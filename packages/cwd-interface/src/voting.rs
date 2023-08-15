use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw2::ContractVersion;
use cwd_macros::{active_query, info_query, proposal_module_query, token_query, voting_query};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[token_query]
#[voting_query]
#[info_query]
#[active_query]
#[proposal_module_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum Query {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct VotingPowerAtHeightResponse {
    pub power: Uint128,
    pub height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TotalPowerAtHeightResponse {
    pub power: Uint128,
    pub height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ClaimsResponse {
    pub power: Uint128,
    pub height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub info: ContractVersion,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IsActiveResponse {
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BondingStatusResponse {
    pub bonding_enabled: bool,
    pub unbondable_abount: Uint128,
    pub height: u64,
}

mod tests {

    /// Make sure the enum has all of the fields we expect. This will
    /// fail to compile if not.
    #[test]
    fn test_macro_expansion() {
        use crate::voting::{
            InfoResponse, IsActiveResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
        };
        use cosmwasm_schema::{cw_serde, QueryResponses};
        use cosmwasm_std::Addr;
        use cwd_macros::{active_query, info_query, token_query, voting_query};

        #[token_query]
        #[voting_query]
        #[info_query]
        #[active_query]
        #[cw_serde]
        #[derive(QueryResponses)]
        enum Query {}

        let query = Query::TokenContract {};

        match query {
            Query::TokenContract {} => (),
            Query::VotingPowerAtHeight { .. } => (),
            Query::TotalPowerAtHeight { .. } => (),
            Query::IsActive {} => (),
            Query::Info {} => (),
        }
    }
}
