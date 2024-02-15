use neutron_sdk::proto_types::neutron::cron::QueryParamsRequest;
use schemars::JsonSchema;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

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
    #[serde(deserialize_with = "deserialize_cron_limit")]
    pub limit: u64,
}

/// Unfortunately, stargate returns a string instead of a number for the
/// limit parameter, so we need to have a custom deserializer for this
/// field.
fn deserialize_cron_limit<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a number")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse::<u64>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

/// The types below are used for querying cron module parameters via stargate.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, ::prost::Message)]
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
