use std::collections::HashMap;
use std::iter::FromIterator;

use chrono::NaiveDateTime;

#[cfg(feature = "bytes")]
use bytes::Bytes;

use crate::datastore::api::value::ValueType;
use crate::datastore::Key;
use crate::error::ConvertError;

#[cfg(feature = "datastore-derive")]
#[doc(hidden)]
pub use google_cloud_derive::{FromValue, IntoValue};

/// A value, as stored in Datastore.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// NULL
    OptionValue(Option<Box<Value>>),
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

impl Value {
    /// Gets the static name of the type of the value.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::OptionValue(_) => "option",
            Value::BooleanValue(_) => "bool",
            Value::IntegerValue(_) => "integer",
            Value::DoubleValue(_) => "double",
            Value::TimestampValue(_) => "timestamp",
            Value::KeyValue(_) => "key",
            Value::StringValue(_) => "string",
            Value::BlobValue(_) => "blob",
            Value::GeoPointValue(_, _) => "geopoint",
            Value::EntityValue(_) => "entity",
            Value::ArrayValue(_) => "array",
        }
    }
}

/// Trait for converting a type to a Datastore value.
pub trait IntoValue {
    /// Converts the type to a Datastore value.
    fn into_value(self) -> Value;
}

/// Trait for mapping a Datastore value to a type.
pub trait FromValue: Sized {
    /// Attempts to construct a value of this type from the passed Datastore value.
    fn from_value(value: Value) -> Result<Self, ConvertError>;
}

impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::StringValue(self)
    }
}

impl IntoValue for &str {
    fn into_value(self) -> Value {
        String::from(self).into_value()
    }
}

impl IntoValue for i8 {
    fn into_value(self) -> Value {
        Value::IntegerValue(self as i64)
    }
}

impl IntoValue for i16 {
    fn into_value(self) -> Value {
        Value::IntegerValue(self as i64)
    }
}

impl IntoValue for i32 {
    fn into_value(self) -> Value {
        Value::IntegerValue(self as i64)
    }
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::IntegerValue(self)
    }
}

impl IntoValue for f32 {
    fn into_value(self) -> Value {
        Value::DoubleValue(self as f64)
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::DoubleValue(self)
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::BooleanValue(self)
    }
}

impl IntoValue for Key {
    fn into_value(self) -> Value {
        Value::KeyValue(self)
    }
}

impl IntoValue for NaiveDateTime {
    fn into_value(self) -> Value {
        Value::TimestampValue(self)
    }
}

impl<T> IntoValue for Option<T> 
where
    T: IntoValue,
{
    fn into_value(self) -> Value {
        Value::OptionValue(
            match self {
                Some(x) => Some(Box::new(x.into_value())),
                None => None
            }
        )
    }
}

#[cfg(feature = "bytes")]
impl IntoValue for Bytes {
    fn into_value(self) -> Value {
        Value::BlobValue(self.to_vec())
    }
}

impl<T> IntoValue for Vec<T>
where
    T: IntoValue,
{
    fn into_value(self) -> Value {
        Value::ArrayValue(self.into_iter().map(IntoValue::into_value).collect())
    }
}

impl<T> IntoValue for HashMap<String, T>
where
    T: IntoValue,
{
    fn into_value(self) -> Value {
        Value::EntityValue(self.into_iter().map(|(k, v)| (k, v.into_value())).collect())
    }
}

impl<T> FromIterator<T> for Value
where
    T: IntoValue,
{
    fn from_iter<I>(iter: I) -> Value
    where
        I: IntoIterator<Item = T>,
    {
        Value::ArrayValue(iter.into_iter().map(IntoValue::into_value).collect())
    }
}

impl FromValue for Value {
    fn from_value(value: Value) -> Result<Value, ConvertError> {
        Ok(value)
    }
}

impl FromValue for String {
    fn from_value(value: Value) -> Result<String, ConvertError> {
        match value {
            Value::StringValue(value) => Ok(value),
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("string"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl FromValue for i64 {
    fn from_value(value: Value) -> Result<i64, ConvertError> {
        match value {
            Value::IntegerValue(value) => Ok(value),
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("integer"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: Value) -> Result<f64, ConvertError> {
        match value {
            Value::DoubleValue(value) => Ok(value),
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("double"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl FromValue for bool {
    fn from_value(value: Value) -> Result<bool, ConvertError> {
        match value {
            Value::BooleanValue(value) => Ok(value),
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("bool"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl FromValue for Key {
    fn from_value(value: Value) -> Result<Key, ConvertError> {
        match value {
            Value::KeyValue(value) => Ok(value),
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("key"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl FromValue for NaiveDateTime {
    fn from_value(value: Value) -> Result<NaiveDateTime, ConvertError> {
        match value {
            Value::TimestampValue(value) => Ok(value),
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("timestamp"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl<T> FromValue for Option<T>
where
    T: FromValue,
{
    fn from_value(value: Value) -> Result<Option<T>, ConvertError> {
        match value.clone() {
            Value::OptionValue(_) => Ok(None),
            _ => Ok(Some(FromValue::from_value(value)?)),
        }
    }
}

#[cfg(feature = "bytes")]
impl FromValue for Bytes {
    fn from_value(value: Value) -> Result<Bytes, ConvertError> {
        match value {
            Value::BlobValue(value) => Ok(Bytes::from(value)),
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("blob"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl<T> FromValue for Vec<T>
where
    T: FromValue,
{
    fn from_value(value: Value) -> Result<Vec<T>, ConvertError> {
        match value {
            Value::ArrayValue(values) => {
                let values = values
                    .into_iter()
                    .map(FromValue::from_value)
                    .collect::<Result<Vec<T>, ConvertError>>()?;
                Ok(values)
            }
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("array"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl<T> FromValue for HashMap<String, T>
where
    T: FromValue,
{
    fn from_value(value: Value) -> Result<HashMap<String, T>, ConvertError> {
        match value {
            Value::EntityValue(values) => {
                let values = values
                    .into_iter()
                    .map(|(k, v)| {
                        let v = FromValue::from_value(v)?;
                        Ok((k, v))
                    })
                    .collect::<Result<HashMap<String, T>, ConvertError>>()?;
                Ok(values)
            }
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("entity"),
                got: String::from(value.type_name()),
            }),
        }
    }
}

impl From<ValueType> for Value {
    fn from(value: ValueType) -> Value {
        match value {
            ValueType::NullValue(_) => Value::OptionValue(None),
            ValueType::BooleanValue(val) => Value::BooleanValue(val),
            ValueType::IntegerValue(val) => Value::IntegerValue(val),
            ValueType::DoubleValue(val) => Value::DoubleValue(val),
            ValueType::TimestampValue(val) => {
                Value::TimestampValue(NaiveDateTime::from_timestamp(val.seconds, val.nanos as u32))
            }
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
