use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
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

pub fn deserialize_u64_vec<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumberVecVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVecVisitor {
        type Value = Vec<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a vector of u64s or strings representing u64s")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();

            while let Some(value) = seq.next_element::<U64OrString>()? {
                match value {
                    U64OrString::U64(num) => vec.push(num),
                    U64OrString::String(s) => {
                        let num: u64 = s.parse().map_err(de::Error::custom)?;
                        vec.push(num);
                    }
                }
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(StringOrNumberVecVisitor)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum U64OrString {
    U64(u64),
    String(String),
}
