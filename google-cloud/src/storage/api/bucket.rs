use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::storage::api::bucket_acl::BucketAclResource;
use crate::storage::api::object_acl::ObjectAclResource;
use crate::deserializer::deserialize_u64_or_string;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketResources {
    /// Value: "storage#buckets"
    pub kind: String,
    #[serde(default)]
    pub items: Vec<BucketResource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketResource {
    /// Value: "storage#bucket"
    pub kind: String,
    pub id: String,
    pub self_link: String,
    pub project_number: String,
    pub name: String,
    pub time_created: String,
    pub updated: String,
    pub default_event_based_hold: Option<bool>,
    pub retention_policy: Option<BucketRetentionPolicy>,
    pub metageneration: String,
    pub acl: Option<Vec<BucketAclResource>>,
    pub default_object_acl: Option<Vec<ObjectAclResource>>,
    pub iam_configuration: Option<BucketIamConfig>,
    pub encryption: Option<BucketEncryption>,
    pub owner: Option<BucketOwner>,
    pub location: String,
    pub location_type: String,
    pub website: Option<BucketWebsite>,
    pub logging: Option<BucketLogging>,
    pub versioning: Option<BucketVersioning>,
    pub cors: Option<Vec<BucketCors>>,
    pub lifecycle: Option<BucketLifecycle>,
    pub labels: Option<HashMap<String, String>>,
    pub storage_class: String,
    pub billing: Option<BucketBilling>,
    pub etag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketRetentionPolicy {
    #[serde(deserialize_with = "deserialize_u64_or_string")]
    pub retention_period: u64,
    pub effective_time: String,
    pub is_locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketIamConfig {
    pub uniform_bucket_level_access: BucketUniformLevelAccess,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketUniformLevelAccess {
    pub enabled: bool,
    pub locked_time: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketEncryption {
    pub default_kms_key_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketOwner {
    pub entity: String,
    pub entity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketWebsite {
    pub main_page_suffix: String,
    pub not_found_page: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketLogging {
    pub log_bucket: String,
    pub log_object_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketVersioning {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketCors {
    pub origin: Vec<String>,
    pub method: Vec<String>,
    pub response_header: Vec<String>,
    pub max_age_seconds: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketLifecycle {
    pub rule: Vec<BucketRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketRule {
    pub action: BucketRuleAction,
    pub condition: BucketRuleCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketRuleAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub storage_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketRuleCondition {
    pub age: i32,
    pub created_before: Option<String>,
    pub is_live: Option<bool>,
    pub matches_storage_class: Option<Vec<String>>,
    pub num_newer_versions: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketBilling {
    pub requester_pays: bool,
}
