use crate::utils::deserialize_u64;
use cosmwasm_std::Coin;
use neutron_sdk::proto_types::osmosis::tokenfactory::v1beta1::QueryParamsRequest;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PARAMS_QUERY_PATH_TOKENFACTORY: &str = "/neutron.tokenfactory.Query/Params";
pub const MSG_TYPE_UPDATE_PARAMS_TOKENFACTORY: &str = "/osmosis.tokenfactory.v1beta1.MsgUpdateParamsResponse";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MsgUpdateParamsTokenfactory {
    pub params: ParamsTokenfactory,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct WhitelistedHook {
    #[serde(deserialize_with = "deserialize_u64")]
    pub code_id: u64,
    pub denom_creator: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamsTokenfactory {
    pub denom_creation_fee: Vec<Coin>,
    #[serde(deserialize_with = "deserialize_u64")]
    pub denom_creation_gas_consume: u64,
    pub fee_collector_address: String,
    pub whitelisted_hooks: Vec<WhitelistedHook>,
}

/// The types below are used for querying tokenfactory module parameters via stargate.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, ::prost::Message)]
pub struct ParamsRequestTokenfactory {}

impl From<ParamsRequestTokenfactory> for QueryParamsRequest {
    fn from(_: ParamsRequestTokenfactory) -> QueryParamsRequest {
        QueryParamsRequest {}
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ParamsResponseTokenfactory {
    pub params: ParamsTokenfactory,
}
