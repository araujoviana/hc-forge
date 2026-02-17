use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NatGateway {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub spec: Option<String>,
    pub status: Option<String>,
    pub router_id: Option<String>,
    pub internal_network_id: Option<String>,
    pub enterprise_project_id: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NatGatewayListResponse {
    #[serde(default)]
    pub nat_gateways: Vec<NatGateway>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NatGatewaySingleResponse {
    #[serde(default)]
    pub nat_gateway: NatGateway,
}

#[derive(Debug, Clone, Serialize)]
pub struct NatGatewayCreateRequest {
    pub nat_gateway: NatGatewayCreateBody,
}

#[derive(Debug, Clone, Serialize)]
pub struct NatGatewayCreateBody {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub spec: String,
    pub router_id: String,
    pub internal_network_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnatRuleCreateRequest {
    pub snat_rule: SnatRuleCreateBody,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnatRuleCreateBody {
    pub nat_gateway_id: String,
    pub network_id: String,
    pub floating_ip_id: String,
}

#[cfg(test)]
mod tests {
    use super::{
        NatGatewayCreateBody, NatGatewayCreateRequest, NatGatewayListResponse, SnatRuleCreateBody,
        SnatRuleCreateRequest,
    };

    #[test]
    fn nat_gateway_create_request_serializes_expected_fields() {
        let payload = NatGatewayCreateRequest {
            nat_gateway: NatGatewayCreateBody {
                name: "cce-nat".to_string(),
                description: Some("for cce egress".to_string()),
                spec: "1".to_string(),
                router_id: "vpc-123".to_string(),
                internal_network_id: "subnet-456".to_string(),
                enterprise_project_id: None,
            },
        };

        let value = serde_json::to_value(payload).expect("serialize nat gateway create payload");
        assert_eq!(value["nat_gateway"]["name"], "cce-nat");
        assert_eq!(value["nat_gateway"]["spec"], "1");
        assert_eq!(value["nat_gateway"]["router_id"], "vpc-123");
        assert_eq!(value["nat_gateway"]["internal_network_id"], "subnet-456");
        assert!(value["nat_gateway"].get("enterprise_project_id").is_none());
    }

    #[test]
    fn nat_gateway_list_response_deserializes_items() {
        let raw = r#"{
          "nat_gateways":[
            {
              "id":"nat-1",
              "name":"cce-nat",
              "spec":"1",
              "status":"ACTIVE",
              "router_id":"vpc-123",
              "internal_network_id":"subnet-456",
              "created_at":"2026-02-18 10:30:00"
            }
          ]
        }"#;

        let response: NatGatewayListResponse =
            serde_json::from_str(raw).expect("deserialize nat gateway list response");
        assert_eq!(response.nat_gateways.len(), 1);
        assert_eq!(response.nat_gateways[0].id.as_deref(), Some("nat-1"));
        assert_eq!(response.nat_gateways[0].status.as_deref(), Some("ACTIVE"));
        assert_eq!(
            response.nat_gateways[0].internal_network_id.as_deref(),
            Some("subnet-456")
        );
    }

    #[test]
    fn snat_rule_create_request_serializes_expected_fields() {
        let payload = SnatRuleCreateRequest {
            snat_rule: SnatRuleCreateBody {
                nat_gateway_id: "nat-1".to_string(),
                network_id: "subnet-1".to_string(),
                floating_ip_id: "eip-1".to_string(),
            },
        };

        let value = serde_json::to_value(payload).expect("serialize snat create payload");
        assert_eq!(value["snat_rule"]["nat_gateway_id"], "nat-1");
        assert_eq!(value["snat_rule"]["network_id"], "subnet-1");
        assert_eq!(value["snat_rule"]["floating_ip_id"], "eip-1");
    }
}
