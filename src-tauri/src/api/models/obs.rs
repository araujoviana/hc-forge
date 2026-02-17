use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsBucket {
    pub name: String,
    pub creation_date: Option<String>,
    pub location: Option<String>,
    pub bucket_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsListBucketsResponse {
    pub buckets: Vec<ObsBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsObject {
    pub key: String,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
    pub size: Option<u64>,
    pub storage_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsListObjectsResponse {
    pub bucket: String,
    pub prefix: Option<String>,
    pub marker: Option<String>,
    pub next_marker: Option<String>,
    pub is_truncated: bool,
    pub objects: Vec<ObsObject>,
}
