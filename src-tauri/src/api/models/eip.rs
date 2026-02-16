use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub current_count: Option<u32>,
    pub next_marker: Option<String>,
    pub previous_marker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EipVnic {
    pub private_ip_address: Option<String>,
    pub device_id: Option<String>,
    pub device_owner: Option<String>,
    pub vpc_id: Option<String>,
    pub port_id: Option<String>,
    pub mac: Option<String>,
    pub vtep: Option<String>,
    pub vni: Option<String>,
    pub instance_id: Option<String>,
    pub instance_type: Option<String>,
    pub port_profile: Option<String>,
    pub port_vif_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EipBandwidth {
    pub id: Option<String>,
    pub size: Option<u32>,
    pub share_type: Option<String>,
    pub charge_mode: Option<String>,
    pub name: Option<String>,
    pub billing_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIp {
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub lock_status: Option<String>,
    #[serde(default)]
    pub allow_share_bandwidth_types: Vec<String>,
    pub id: Option<String>,
    pub alias: Option<String>,
    pub project_id: Option<String>,
    pub ip_version: Option<u32>,
    pub public_ip_address: Option<String>,
    pub public_ipv6_address: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub enterprise_project_id: Option<String>,
    pub billing_info: Option<String>,
    #[serde(rename = "type")]
    pub eip_type: Option<String>,
    pub vnic: Option<EipVnic>,
    pub bandwidth: Option<EipBandwidth>,
    pub associate_instance_type: Option<String>,
    pub associate_instance_id: Option<String>,
    pub publicip_pool_id: Option<String>,
    pub publicip_pool_name: Option<String>,
    pub public_border_group: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EipListResponse {
    pub page_info: Option<PageInfo>,
    #[serde(default)]
    pub publicips: Vec<PublicIp>,
    pub request_id: Option<String>,
    pub total_count: Option<u32>,
}
