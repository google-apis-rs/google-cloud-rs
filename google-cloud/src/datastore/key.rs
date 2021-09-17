use std::borrow::Borrow;

use crate::datastore::api;
use crate::datastore::api::key::path_element::IdType;

/// Represents a key's ID.
///
/// It can either be a integer key, a string/named key or an incomplete key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyID {
    /// A string/named key ID.
    StringID(String),
    /// An integer key ID.
    IntID(i64),
    /// An incomplete key ID (unspecified ID).
    Incomplete,
}

impl KeyID {
    /// Is this ID incomplete ?
    ///
    /// ```
    /// # use google_cloud::datastore::KeyID;
    /// let id1 = KeyID::Incomplete;
    /// let id2 = KeyID::IntID(10);
    /// assert!(id1.is_incomplete());
    /// assert!(!id2.is_incomplete());
    /// ```
    pub fn is_incomplete(&self) -> bool {
        matches!(self, KeyID::Incomplete)
    }
}

impl From<i64> for KeyID {
    fn from(id: i64) -> KeyID {
        KeyID::IntID(id)
    }
}

impl From<&str> for KeyID {
    fn from(id: &str) -> KeyID {
        KeyID::from(String::from(id))
    }
}

impl From<String> for KeyID {
    fn from(id: String) -> KeyID {
        KeyID::StringID(id)
    }
}

impl From<IdType> for KeyID {
    fn from(id_type: IdType) -> KeyID {
        match id_type {
            IdType::Id(id) => KeyID::IntID(id),
            IdType::Name(id) => KeyID::StringID(id),
        }
    }
}

/// Represents an entity's key.
///
/// ```
/// # use google_cloud::datastore::Key;
/// let key = Key::new("kind").id("entity-name");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub(crate) kind: String,
    pub(crate) id: KeyID,
    pub(crate) parent: Option<Box<Key>>,
    pub(crate) namespace: Option<String>,
}

impl Key {
    /// Create a new incomplete key.
    ///
    /// ```
    /// # use google_cloud::datastore::Key;
    /// let incomplete_key = Key::new("kind");
    /// ```
    pub fn new(kind: impl Into<String>) -> Key {
        Key {
            kind: kind.into(),
            id: KeyID::Incomplete,
            parent: None,
            namespace: None,
        }
    }

    /// Get the key's kind.
    ///
    /// ```
    /// # use google_cloud::datastore::Key;
    /// let key = Key::new("kind").id(10);
    /// assert_eq!(key.get_kind(), "kind");
    /// ```
    pub fn get_kind(&self) -> &str {
        self.kind.as_str()
    }

    /// Attach an ID to the key.
    ///
    /// ```
    /// # use google_cloud::datastore::Key;
    /// let int_key = Key::new("kind").id(10);
    /// let string_key = Key::new("kind").id("entity-name");
    /// ```
    pub fn id(mut self, id: impl Into<KeyID>) -> Key {
        self.id = id.into();
        self
    }

    /// Get the key's ID.
    ///
    /// ```
    /// # use google_cloud::datastore::{Key, KeyID};
    /// let key = Key::new("kind").id(10);
    /// assert_eq!(key.get_id(), &KeyID::IntID(10));
    /// ```
    pub fn get_id(&self) -> &KeyID {
        &self.id
    }

    /// Attach an ancestor key to the key.
    ///
    /// ```
    /// # use google_cloud::datastore::Key;
    /// let ancestor = Key::new("kind").id(10);
    /// let key = Key::new("kind").parent(ancestor);
    /// ```
    pub fn parent(mut self, parent: impl Into<Box<Key>>) -> Key {
        self.parent = Some(parent.into());
        self
    }

    /// Get the key's ancestor key, if any.
    ///
    /// ```
    /// # use google_cloud::datastore::Key;
    /// let ancestor = Key::new("kind").id(10);
    /// let key = Key::new("kind").id("name").parent(ancestor.clone());
    /// assert_eq!(key.get_parent(), Some(&ancestor));
    /// assert_eq!(ancestor.get_parent(), None);
    /// ```
    pub fn get_parent(&self) -> Option<&Key> {
        self.parent.as_ref().map(|inner| inner.borrow())
    }

    /// Attach a namespace to the key (for multitenancy purposes).
    ///
    /// ```
    /// # use google_cloud::datastore::Key;
    /// let key = Key::new("kind").namespace("dev");
    /// ```
    pub fn namespace(mut self, namespace: impl Into<String>) -> Key {
        self.namespace = Some(namespace.into());
        self
    }

    /// Get the key's namespace, if any.
    ///
    /// ```
    /// # use google_cloud::datastore::Key;
    /// let key1 = Key::new("kind").id(10);
    /// let key2 = Key::new("kind").namespace("dev").id(10);
    /// assert_eq!(key1.get_namespace(), None);
    /// assert_eq!(key2.get_namespace(), Some("dev"));
    /// ```
    pub fn get_namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// Is the key incomplete (missing an ID) ?
    ///
    /// ```
    /// # use google_cloud::datastore::Key;
    /// let key1 = Key::new("kind").namespace("dev");
    /// let key2 = Key::new("kind").id(10);
    /// assert!(key1.is_incomplete());
    /// assert!(!key2.is_incomplete());
    /// ```
    pub fn is_incomplete(&self) -> bool {
        self.get_id().is_incomplete()
    }
}

impl From<api::Key> for Key {
    fn from(key: api::Key) -> Key {
        let data = key.partition_id.unwrap();
        let key = key.path.into_iter().fold(None, |acc, el| {
            let key_id = match el.id_type {
                None => KeyID::Incomplete,
                Some(id_type) => KeyID::from(id_type),
            };
            let key = Key::new(el.kind);
            let key = if data.namespace_id.is_empty() {
                key
            } else {
                key.namespace(data.namespace_id.as_str())
            };
            let key = key.id(key_id);

            if let Some(ancestor) = acc {
                Some(key.parent(ancestor))
            } else {
                Some(key)
            }
        });

        //? There should always be at least one.
        key.unwrap()
    }
}
