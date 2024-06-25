use serde::de::{self, Visitor};
use serde::Deserializer;
use std::fmt;

/// Unfortunately, stargate returns a string instead of a number for
/// u64 parameters, so we need to have a custom deserializer for these
/// field.
pub fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
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
