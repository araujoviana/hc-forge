use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

fn deserialize_u32_opt<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    let number = match value {
        None => None,
        Some(serde_json::Value::Number(num)) => {
            num.as_u64().and_then(|val| u32::try_from(val).ok())
        }
        Some(serde_json::Value::String(text)) => text.parse::<u32>().ok(),
        _ => None,
    };
    Ok(number)
}

#[derive(Serialize)]
pub struct CreateEcsRequest {
    pub server: Server,
}

#[derive(Serialize)]
pub struct Server {
    pub name: String,

    #[serde(rename = "imageRef")]
    pub image_ref: String,

    #[serde(rename = "flavorRef")]
    pub flavor_ref: String,

    pub vpcid: String,
    pub nics: Vec<Nic>,

    #[serde(rename = "root_volume")]
    pub root_volume: RootVolume,

    #[serde(
        rename = "data_volumes",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub data_volumes: Vec<DataVolume>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publicip: Option<PublicIp>,

    #[serde(rename = "adminPass", skip_serializing_if = "Option::is_none")]
    pub admin_pass: Option<String>,
}

#[derive(Serialize)]
pub struct Nic {
    pub subnet_id: String,
}

#[derive(Serialize)]
pub struct RootVolume {
    pub volumetype: String,
    pub size: u32,
}

#[derive(Serialize)]
pub struct DataVolume {
    pub volumetype: String,
    pub size: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiattach: Option<bool>,

    #[serde(rename = "hw:passthrough", skip_serializing_if = "Option::is_none")]
    pub hw_passthrough: Option<bool>,
}

#[derive(Serialize)]
pub struct PublicIp {
    pub eip: Eip,
}

#[derive(Serialize)]
pub struct Eip {
    #[serde(rename = "iptype")]
    pub ip_type: String,
    pub bandwidth: Bandwidth,
}

#[derive(Serialize)]
pub struct Bandwidth {
    pub size: u32,

    #[serde(rename = "sharetype")]
    pub share_type: String,

    #[serde(rename = "chargemode")]
    pub charge_mode: String,
}

#[derive(Serialize)]
pub struct DeleteEcsRequest {
    pub servers: Vec<DeleteEcsServer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_publicip: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_volume: Option<bool>,
}

#[derive(Serialize)]
pub struct DeleteEcsServer {
    pub id: String,
}

#[derive(Serialize)]
pub struct StopEcsRequest {
    #[serde(rename = "os-stop")]
    pub os_stop: StopEcsAction,
}

#[derive(Serialize)]
pub struct StopEcsAction {
    pub servers: Vec<StopEcsServer>,
    #[serde(rename = "type")]
    pub stop_type: String,
}

#[derive(Serialize)]
pub struct StopEcsServer {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flavor {
    pub id: String,
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_u32_opt")]
    pub vcpus: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_opt")]
    pub ram: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_opt")]
    pub disk: Option<u32>,
    #[serde(default)]
    pub os_extra_specs: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlavorListResponse {
    pub flavors: Vec<Flavor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsFlavorInfo {
    pub disk: Option<u32>,
    pub name: Option<String>,
    pub id: Option<String>,
    pub vcpus: Option<u32>,
    pub ram: Option<u32>,
    #[serde(default)]
    pub gpus: Vec<serde_json::Value>,
    #[serde(default)]
    pub asic_accelerators: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsServer {
    pub tenant_id: Option<String>,
    pub vm_state: Option<String>,
    pub flavor: Option<EcsFlavorInfo>,
    pub availability_zone: Option<String>,
    pub user_id: Option<String>,
    pub created: Option<String>,
    pub name: Option<String>,
    pub task_state: Option<String>,
    pub in_recycle_bin: Option<bool>,
    pub id: Option<String>,
    pub updated: Option<String>,
    pub spod_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsServerLink {
    pub rel: Option<String>,
    pub href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsListResponse {
    #[serde(default)]
    pub servers: Vec<EcsServer>,
    #[serde(default, rename = "servers_links")]
    pub servers_links: Vec<EcsServerLink>,
    pub request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_serializes_admin_pass_only_when_present() {
        let without_password = Server {
            name: "example".to_string(),
            image_ref: "img".to_string(),
            flavor_ref: "flavor".to_string(),
            vpcid: "vpc".to_string(),
            nics: vec![Nic {
                subnet_id: "subnet".to_string(),
            }],
            root_volume: RootVolume {
                volumetype: "GPSSD".to_string(),
                size: 40,
            },
            data_volumes: Vec::new(),
            publicip: None,
            admin_pass: None,
        };
        let without_json =
            serde_json::to_string(&without_password).expect("serialize server without password");
        assert!(!without_json.contains("adminPass"));

        let with_password = Server {
            admin_pass: Some("Passw0rd!".to_string()),
            ..without_password
        };
        let with_json =
            serde_json::to_string(&with_password).expect("serialize server with password");
        assert!(with_json.contains("\"adminPass\":\"Passw0rd!\""));
    }

    #[test]
    fn delete_request_serializes_expected_fields() {
        let payload = DeleteEcsRequest {
            servers: vec![DeleteEcsServer {
                id: "server-id".to_string(),
            }],
            delete_publicip: Some(true),
            delete_volume: Some(true),
        };
        let value =
            serde_json::to_value(payload).expect("serialize delete ecs payload to json value");
        assert_eq!(value["servers"][0]["id"], "server-id");
        assert_eq!(value["delete_publicip"], true);
        assert_eq!(value["delete_volume"], true);
    }

    #[test]
    fn stop_request_serializes_expected_fields() {
        let payload = StopEcsRequest {
            os_stop: StopEcsAction {
                servers: vec![StopEcsServer {
                    id: "server-id".to_string(),
                }],
                stop_type: "SOFT".to_string(),
            },
        };
        let value =
            serde_json::to_value(payload).expect("serialize stop ecs payload to json value");
        assert_eq!(value["os-stop"]["servers"][0]["id"], "server-id");
        assert_eq!(value["os-stop"]["type"], "SOFT");
    }
}
