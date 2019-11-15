use std::collections::HashMap;

use crate::datastore::api;
use crate::datastore::{Key, Value};

/// Represents a Datastore entity.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub(crate) key: Key,
    pub(crate) properties: HashMap<String, Value>,
}

impl Entity {
    /// Constructs a new Entity.
    pub fn new(key: Key, properties: HashMap<String, Value>) -> Entity {
        Entity { key, properties }
    }

    /// Get the entity's key.
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Get the entity's properties.
    pub fn properties(&self) -> &HashMap<String, Value> {
        &self.properties
    }

    /// Get a mutable reference to the entity's properties.
    pub fn properties_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.properties
    }
}

impl From<api::Entity> for Entity {
    fn from(entity: api::Entity) -> Entity {
        let key = Key::from(entity.key.unwrap());
        let properties = entity
            .properties
            .into_iter()
            .map(|(k, v)| (k, Value::from(v.value_type.unwrap())))
            .collect();

        Entity { key, properties }
    }
}
