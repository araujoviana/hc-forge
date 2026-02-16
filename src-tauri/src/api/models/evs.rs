use serde::{Deserialize, Deserializer, Serialize};

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

fn deserialize_bool_opt<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    let parsed = match value {
        None => None,
        Some(serde_json::Value::Bool(flag)) => Some(flag),
        Some(serde_json::Value::Number(num)) => num.as_u64().map(|raw| raw != 0),
        Some(serde_json::Value::String(text)) => {
            let normalized = text.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "true" | "yes" | "y" | "1" => Some(true),
                "false" | "no" | "n" | "0" => Some(false),
                _ => None,
            }
        }
        _ => None,
    };
    Ok(parsed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvsAttachment {
    pub id: Option<String>,
    pub server_id: Option<String>,
    pub device: Option<String>,
    pub attached_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvsVolume {
    pub id: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_u32_opt")]
    pub size: Option<u32>,
    pub volume_type: Option<String>,
    pub availability_zone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_opt")]
    pub bootable: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_opt")]
    pub multiattach: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub attachments: Vec<EvsAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvsListResponse {
    #[serde(default)]
    pub volumes: Vec<EvsVolume>,
    #[serde(default, deserialize_with = "deserialize_u32_opt")]
    pub count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::{EvsListResponse, EvsVolume};

    #[test]
    fn evs_volume_deserializes_mixed_types() {
        let raw = r#"{
          "id": "vol-1",
          "size": "100",
          "bootable": "true",
          "multiattach": 0
        }"#;

        let volume: EvsVolume = serde_json::from_str(raw).expect("deserialize evs volume");
        assert_eq!(volume.id.as_deref(), Some("vol-1"));
        assert_eq!(volume.size, Some(100));
        assert_eq!(volume.bootable, Some(true));
        assert_eq!(volume.multiattach, Some(false));
    }

    #[test]
    fn evs_list_response_deserializes_string_count() {
        let raw = r#"{"count":"2","volumes":[]}"#;
        let body: EvsListResponse =
            serde_json::from_str(raw).expect("deserialize evs list response");
        assert_eq!(body.count, Some(2));
        assert!(body.volumes.is_empty());
    }
}
