use crate::utils::{deserialize_u64, deserialize_u64_vec};
use neutron_sdk::proto_types::neutron::dex::QueryParamsRequest;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PARAMS_QUERY_PATH_DEX: &str = "/neutron.dex.Query/Params";
pub const MSG_TYPE_UPDATE_PARAMS_DEX: &str = "/neutron.dex.MsgUpdateParams";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MsgUpdateParamsDex {
    pub params: ParamsDex,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Unfortunately, stargate returns a string instead of a number for the
/// u64 fields, so we need to have a custom deserializer.
pub struct ParamsDex {
    #[serde(deserialize_with = "deserialize_u64_vec")]
    pub fee_tiers: Vec<u64>,
    pub paused: bool,
    #[serde(deserialize_with = "deserialize_u64")]
    pub max_jits_per_block: u64,
    #[serde(deserialize_with = "deserialize_u64")]
    pub good_til_purge_allowance: u64,
}

/// The types below are used for querying dex module parameters via stargate.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, ::prost::Message)]
pub struct ParamsRequestDex {}

impl From<ParamsRequestDex> for QueryParamsRequest {
    fn from(_: ParamsRequestDex) -> QueryParamsRequest {
        QueryParamsRequest {}
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ParamsResponseDex {
    pub params: ParamsDex,
}
