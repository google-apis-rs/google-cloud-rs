use crate::datastore::api;
use crate::datastore::{IntoValue, Key, Value};
use crate::error::ConvertError;

/// Represents a Datastore entity.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub(crate) key: Key,
    pub(crate) properties: Value,
}

impl Entity {
    /// Constructs a new Entity.
    pub fn new(key: Key, value: impl IntoValue) -> Result<Entity, ConvertError> {
        let properties = value.into_value();
        match properties {
            Value::EntityValue(_) => Ok(Entity { key, properties }),
            _ => Err(ConvertError::UnexpectedPropertyType {
                expected: String::from("entity"),
                got: String::from(properties.type_name()),
            }),
        }
    }

    /// Get the entity's key.
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Get the entity's properties.
    pub fn properties(&self) -> &Value {
        &self.properties
    }

    /// Get a mutable reference to the entity's properties.
    pub fn properties_mut(&mut self) -> &mut Value {
        &mut self.properties
    }
}

/// Trait for converting a type to a Datastore entity (key + value).
pub trait IntoEntity {
    /// Attempts to construct a value of this type from the passed Datastore value.
    /// Fails if the top level value is not a `Value::EntityValue`.
    fn into_entity(self) -> Result<Entity, ConvertError>;
}

impl IntoEntity for Entity {
    fn into_entity(self) -> Result<Entity, ConvertError> {
        Ok(self)
    }
}

impl<V> IntoEntity for (Key, V)
where
    V: IntoValue,
{
    fn into_entity(self) -> Result<Entity, ConvertError> {
        let (k, v) = self;
        Entity::new(k, v)
    }
}

impl From<api::Entity> for Entity {
    fn from(entity: api::Entity) -> Entity {
        let key = Key::from(entity.key.unwrap());
        let properties = entity.properties;

        let properties = properties
            .into_iter()
            .map(|(k, v)| (k, Value::from(v.value_type.unwrap())))
            .collect();
        let properties = Value::EntityValue(properties);

        Entity { key, properties }
    }
}
