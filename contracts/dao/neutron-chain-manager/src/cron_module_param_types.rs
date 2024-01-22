use neutron_sdk::proto_types::neutron::cron::QueryParamsRequest;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PARAMS_QUERY_PATH_CRON: &str = "/neutron.cron.Query/Params";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MsgUpdateParamsCron {
    pub params: ParamsCron,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamsCron {
    pub security_address: String,
    pub limit: u64,
}

/// The types below are used for querying cron module parameters via stargate.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ParamsRequestCron {}

impl From<ParamsRequestCron> for QueryParamsRequest {
    fn from(_: ParamsRequestCron) -> QueryParamsRequest {
        QueryParamsRequest {}
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ParamsResponseCron {
    pub params: ParamsCron,
}
