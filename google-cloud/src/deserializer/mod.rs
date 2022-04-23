use std::fmt;
use serde::de::{self, Unexpected};
use serde::Deserializer;

pub fn deserialize_u64_or_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
{
    deserializer.deserialize_any(DeserializeU64OrStringVisitor)
}

struct DeserializeU64OrStringVisitor;
impl<'de> de::Visitor<'de> for DeserializeU64OrStringVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a string")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        Ok(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        if let Ok(n) = v.parse::<u64>() {
            Ok(n)
        } else if v.is_empty() {
            Ok(0)
        } else {
            Err(E::invalid_value(Unexpected::Str(v), &self))
        }
    }
}
