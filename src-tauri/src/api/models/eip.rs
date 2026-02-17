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

#[derive(Debug, Clone, Serialize)]
pub struct CreatePublicIpRequest {
    pub publicip: CreatePublicIpBody,
    pub bandwidth: CreatePublicIpBandwidth,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePublicIpBody {
    #[serde(rename = "type")]
    pub ip_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePublicIpBandwidth {
    pub name: String,
    pub size: u32,
    pub share_type: String,
    pub charge_mode: String,
}

#[cfg(test)]
mod tests {
    use super::{CreatePublicIpBandwidth, CreatePublicIpBody, CreatePublicIpRequest};

    #[test]
    fn create_public_ip_request_serializes_expected_fields() {
        let payload = CreatePublicIpRequest {
            publicip: CreatePublicIpBody {
                ip_type: "5_bgp".to_string(),
            },
            bandwidth: CreatePublicIpBandwidth {
                name: "cce-nat-eip".to_string(),
                size: 100,
                share_type: "PER".to_string(),
                charge_mode: "traffic".to_string(),
            },
        };

        let value = serde_json::to_value(payload).expect("serialize create public ip payload");
        assert_eq!(value["publicip"]["type"], "5_bgp");
        assert_eq!(value["bandwidth"]["name"], "cce-nat-eip");
        assert_eq!(value["bandwidth"]["size"], 100);
        assert_eq!(value["bandwidth"]["share_type"], "PER");
        assert_eq!(value["bandwidth"]["charge_mode"], "traffic");
    }
}
