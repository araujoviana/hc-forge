use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Vpc {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct VpcListResponse {
    pub vpcs: Vec<Vpc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subnet {
    pub id: String,
    pub name: String,
    pub cidr: String,
    #[serde(default)]
    pub availability_zone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubnetListResponse {
    pub subnets: Vec<Subnet>,
}

// Display formatting removed: it was only needed for CLI-based selection.

#[cfg(test)]
mod tests {
    use super::Subnet;

    #[test]
    fn subnet_deserializes_with_availability_zone() {
        let raw = r#"{"id":"subnet-1","name":"main","cidr":"10.0.0.0/24","availability_zone":"sa-brazil-1a"}"#;
        let subnet: Subnet = serde_json::from_str(raw).expect("deserialize subnet");
        assert_eq!(subnet.id, "subnet-1");
        assert_eq!(subnet.availability_zone.as_deref(), Some("sa-brazil-1a"));
    }

    #[test]
    fn subnet_deserializes_without_availability_zone() {
        let raw = r#"{"id":"subnet-2","name":"secondary","cidr":"10.0.1.0/24"}"#;
        let subnet: Subnet = serde_json::from_str(raw).expect("deserialize subnet");
        assert_eq!(subnet.id, "subnet-2");
        assert!(subnet.availability_zone.is_none());
    }
}
