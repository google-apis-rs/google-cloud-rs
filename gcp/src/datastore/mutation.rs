use crate::datastore::api;
use crate::datastore::{Key, Value};

/// Represents a Datastore mutation operation.
#[derive(Debug, Clone, PartialEq)]
pub enum Mutation {
    Insert(Entity),
    Update(Entity),
    Upsert(Entity),
    Delete(Key),
}

impl Mutation {}
