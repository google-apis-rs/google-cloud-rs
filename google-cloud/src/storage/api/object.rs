use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::storage::api::object_acl::ObjectAclResource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectResource {
    // Value: "storage#object"
    pub kind: String,
    pub id: String,
    pub self_link: String,
    pub name: String,
    pub bucket: String,
    pub generation: i64,
    pub metageneration: i64,
    pub content_type: String,
    pub time_created: String,
    pub updated: String,
    pub time_deleted: String,
    pub temporary_hold: bool,
    pub event_based_hold: bool,
    pub retention_expiration_time: String,
    pub storage_class: String,
    pub time_storage_class_updated: String,
    pub size: u64,
    pub md5_hash: String,
    pub media_link: String,
    pub content_encoding: String,
    pub content_disposition: String,
    pub content_language: String,
    pub cache_control: String,
    pub metadata: HashMap<String, String>,
    pub acl: Vec<ObjectAclResource>,
    pub owner: ObjectOwner,
    pub crc32c: String,
    pub component_count: i32,
    pub etag: String,
    pub customer_encryption: ObjectCustomerEncryption,
    pub kms_key_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectOwner {
    pub entity: String,
    pub entity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectCustomerEncryption {
    pub encryption_algorithm: String,
    pub key_sha256: String,
}
