use std::collections::HashMap;

use crate::datastore::api::value::ValueType;
use crate::datastore::Key;

/// A value, as stored in Datastore.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A boolean value (true or false).
    BooleanValue(bool),
    /// An integer value.
    IntegerValue(i64),
    /// A floating-point value.
    DoubleValue(f64),
    /// A timestamp value.
    TimestampValue(chrono::NaiveDateTime),
    /// A key value.
    KeyValue(Key),
    /// A string value.
    StringValue(String),
    /// A blob value, just a block of bytes.
    BlobValue(Vec<u8>),
    /// An Earth geographic location value (with latitude and longitude).
    GeoPointValue(f64, f64),
    /// An entity value.
    EntityValue(HashMap<String, Value>),
    /// An array of values.
    ArrayValue(Vec<Value>),
}

impl From<String> for Value {
    fn from(value: String) -> Value {
        Value::StringValue(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Value {
        Value::from(String::from(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Value {
        Value::IntegerValue(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Value {
        Value::DoubleValue(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Value {
        Value::BooleanValue(value)
    }
}

impl From<Key> for Value {
    fn from(value: Key) -> Value {
        Value::KeyValue(value)
    }
}

impl From<ValueType> for Value {
    fn from(value: ValueType) -> Value {
        match value {
            ValueType::NullValue(_) => unreachable!(),
            ValueType::BooleanValue(val) => Value::BooleanValue(val),
            ValueType::IntegerValue(val) => Value::IntegerValue(val),
            ValueType::DoubleValue(val) => Value::DoubleValue(val),
            ValueType::TimestampValue(val) => Value::TimestampValue(
                chrono::NaiveDateTime::from_timestamp(val.seconds, val.nanos as u32),
            ),
            ValueType::KeyValue(key) => Value::KeyValue(Key::from(key)),
            ValueType::StringValue(val) => Value::StringValue(val),
            ValueType::BlobValue(val) => Value::BlobValue(val),
            ValueType::GeoPointValue(val) => Value::GeoPointValue(val.latitude, val.longitude),
            ValueType::EntityValue(entity) => Value::EntityValue({
                entity
                    .properties
                    .into_iter()
                    .map(|(k, v)| (k, Value::from(v.value_type.unwrap())))
                    .collect()
            }),
            ValueType::ArrayValue(seq) => Value::ArrayValue(
                seq.values
                    .into_iter()
                    .map(|val| Value::from(val.value_type.unwrap()))
                    .collect(),
            ),
        }
    }
}
