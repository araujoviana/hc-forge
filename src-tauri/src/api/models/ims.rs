use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "__whole_image", alias = "whole_image")]
    pub whole_image: Option<bool>,
    pub id: String,
    pub name: String,
    pub status: String,
    pub visibility: Option<String>,
    pub min_disk: Option<u32>,
    pub min_ram: Option<u32>,
    pub size: Option<u64>,
    pub disk_format: Option<String>,
    pub container_format: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(rename = "__os_version")]
    pub os_version: Option<String>,
    #[serde(rename = "__os_type")]
    pub os_type: Option<String>,
    #[serde(rename = "__platform")]
    pub platform: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub protected: Option<bool>,
}

// Define a response struct if needed
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageListResponse {
    pub images: Vec<Image>,
}
