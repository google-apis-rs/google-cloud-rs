// use std::fmt;

// use serde::de::{self, DeserializeOwned, Visitor};
// use serde::ser;
// use serde::{Deserialize, Serialize};
// use thiserror::Error;

// use crate::datastore::api;
// use crate::datastore::api::value::ValueType;

// #[derive(Debug, Clone, PartialEq, Error)]
// pub enum SerializeError {
//     #[error("test error")]
//     Test,
// }

// #[derive(Debug, Clone, PartialEq, Error)]
// pub enum DeserializeError {
//     #[error("test error")]
//     Test,
// }

// impl ser::Error for SerializeError {
//     fn custom<T>(msg: T) -> SerializeError
//     where
//         T: fmt::Display,
//     {
//         SerializeError::Test
//     }
// }

// impl de::Error for DeserializeError {
//     fn custom<T>(msg: T) -> DeserializeError
//     where
//         T: fmt::Display,
//     {
//         DeserializeError::Test
//     }
// }

// struct Serializer;
// struct Deserializer {
//     value: api::Value,
// }

// impl<'de> de::IntoDeserializer<'de, DeserializeError> for api::Value {
//     type Deserializer = Deserializer;
//     fn into_deserializer(self) -> Self::Deserializer {
//         Deserializer { value: self }
//     }
// }

// pub fn encode(value: &impl Serialize) -> Result<api::Value, SerializeError> {
//     let mut serializer = Serializer;
//     value.serialize(serializer)
// }

// pub fn decode<T>(value: api::Value) -> Result<T, DeserializeError>
// where
//     T: DeserializeOwned,
// {
//     let mut deserializer = Deserializer { value };
//     T::deserialize(deserializer)
// }

// impl<'a> ser::Serializer for Serializer {
//     type Ok = api::Value;
//     type Error = SerializeError;
// }

// impl<'de, 'a> de::Deserializer<'de> for Deserializer {
//     type Error = DeserializeError;

//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, DeserializeError>
//     where
//         V: Visitor<'de>,
//     {
//         match self.value.value_type {
//             Some(ValueType::NullValue(val)) => visitor.visit_unit(),
//             Some(ValueType::BooleanValue(val)) => visitor.visit_bool(val),
//             Some(ValueType::IntegerValue(val)) => visitor.visit_i64(val),
//             Some(ValueType::DoubleValue(val)) => visitor.visit_f64(val),
//             Some(ValueType::TimestampValue(timestamp)) => unimplemented!(),
//             Some(ValueType::KeyValue(key)) => unimplemented!(),
//             Some(ValueType::StringValue(val)) => visitor.visit_string(val),
//             Some(ValueType::BlobValue(bytes)) => visitor.visit_byte_buf(bytes),
//             Some(ValueType::GeoPointValue(coords)) => unimplemented!(),
//             Some(ValueType::EntityValue(entity)) => unimplemented!(),
//             Some(ValueType::ArrayValue(seq)) => {
//                 let mut seq = de::value::SeqDeserializer::new(seq.values.into_iter());
//                 let ret = visitor.visit_seq(&mut seq)?;
//                 seq.end();
//                 Ok(ret)
//             }
//         }
//     }
// }
