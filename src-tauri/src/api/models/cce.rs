use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct CceCreateClusterRequest {
    pub kind: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub metadata: CceClusterCreateMetadata,
    pub spec: CceClusterCreateSpec,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceClusterCreateMetadata {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceClusterCreateSpec {
    #[serde(rename = "type")]
    pub cluster_type: String,
    pub flavor: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "hostNetwork")]
    pub host_network: CceHostNetwork,
    #[serde(rename = "containerNetwork")]
    pub container_network: CceContainerNetwork,
    #[serde(rename = "kubernetesSvcIpRange")]
    pub kubernetes_svc_ip_range: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<CceAuthentication>,
    #[serde(rename = "clusterTags", skip_serializing_if = "Vec::is_empty", default)]
    pub cluster_tags: Vec<CceClusterTag>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceHostNetwork {
    pub vpc: String,
    pub subnet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceContainerNetwork {
    pub mode: String,
    pub cidr: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceAuthentication {
    pub mode: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceClusterTag {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CceCluster {
    pub kind: Option<String>,
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub spec: Value,
    #[serde(default)]
    pub status: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CceClusterListResponse {
    pub kind: Option<String>,
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
    #[serde(default)]
    pub items: Vec<CceCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CceNodePool {
    pub kind: Option<String>,
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub spec: Value,
    #[serde(default)]
    pub status: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CceNodePoolListResponse {
    pub kind: Option<String>,
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
    #[serde(default)]
    pub items: Vec<CceNodePool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceUpdateClusterRequest {
    pub spec: CceUpdateClusterSpec,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceUpdateClusterSpec {
    #[serde(rename = "clusterExternalIP")]
    pub cluster_external_ip: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceClusterCertRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        CceAuthentication, CceClusterCertRequest, CceClusterCreateMetadata, CceClusterCreateSpec,
        CceClusterTag, CceContainerNetwork, CceCreateClusterRequest, CceHostNetwork,
        CceNodePoolListResponse, CceUpdateClusterRequest, CceUpdateClusterSpec,
    };

    #[test]
    fn cce_create_request_serializes_expected_fields() {
        let payload = CceCreateClusterRequest {
            kind: "Cluster".to_string(),
            api_version: "v3".to_string(),
            metadata: CceClusterCreateMetadata {
                name: "dev-cluster".to_string(),
            },
            spec: CceClusterCreateSpec {
                cluster_type: "VirtualMachine".to_string(),
                flavor: "cce.s2.small".to_string(),
                version: "v1.29".to_string(),
                description: Some("integration cluster".to_string()),
                host_network: CceHostNetwork {
                    vpc: "vpc-id".to_string(),
                    subnet: "subnet-id".to_string(),
                },
                container_network: CceContainerNetwork {
                    mode: "overlay_l2".to_string(),
                    cidr: "172.16.0.0/16".to_string(),
                },
                kubernetes_svc_ip_range: "10.247.0.0/16".to_string(),
                authentication: Some(CceAuthentication {
                    mode: "rbac".to_string(),
                }),
                cluster_tags: Vec::new(),
            },
        };

        let value =
            serde_json::to_value(payload).expect("serialize cce create cluster request to value");

        assert_eq!(value["kind"], "Cluster");
        assert_eq!(value["apiVersion"], "v3");
        assert_eq!(value["metadata"]["name"], "dev-cluster");
        assert_eq!(value["spec"]["type"], "VirtualMachine");
        assert_eq!(value["spec"]["hostNetwork"]["vpc"], "vpc-id");
        assert_eq!(value["spec"]["containerNetwork"]["mode"], "overlay_l2");
        assert_eq!(value["spec"]["kubernetesSvcIpRange"], "10.247.0.0/16");
        assert_eq!(value["spec"]["authentication"]["mode"], "rbac");
    }

    #[test]
    fn cce_list_response_deserializes_items() {
        let raw = r#"{
          "kind":"ClusterList",
          "apiVersion":"v3",
          "items":[
            {
              "kind":"Cluster",
              "apiVersion":"v3",
              "metadata":{"id":"cluster-id","name":"cluster-name"},
              "spec":{"version":"v1.29"},
              "status":{"phase":"Available"}
            }
          ]
        }"#;

        let body: super::CceClusterListResponse =
            serde_json::from_str(raw).expect("deserialize cce cluster list response");
        assert_eq!(body.items.len(), 1);
        assert_eq!(body.kind.as_deref(), Some("ClusterList"));
        assert_eq!(body.items[0].kind.as_deref(), Some("Cluster"));
        assert_eq!(body.items[0].metadata["name"], "cluster-name");
        assert_eq!(body.items[0].status["phase"], "Available");
    }

    #[test]
    fn cce_create_request_omits_optional_fields_when_none() {
        let payload = CceCreateClusterRequest {
            kind: "Cluster".to_string(),
            api_version: "v3".to_string(),
            metadata: CceClusterCreateMetadata {
                name: "minimal-cluster".to_string(),
            },
            spec: CceClusterCreateSpec {
                cluster_type: "VirtualMachine".to_string(),
                flavor: "cce.s2.small".to_string(),
                version: "v1.30".to_string(),
                description: None,
                host_network: CceHostNetwork {
                    vpc: "vpc-id".to_string(),
                    subnet: "subnet-id".to_string(),
                },
                container_network: CceContainerNetwork {
                    mode: "overlay_l2".to_string(),
                    cidr: "172.16.0.0/16".to_string(),
                },
                kubernetes_svc_ip_range: "10.247.0.0/16".to_string(),
                authentication: None,
                cluster_tags: Vec::new(),
            },
        };

        let value =
            serde_json::to_value(payload).expect("serialize cce create request without optionals");
        assert!(value["spec"].get("description").is_none());
        assert!(value["spec"].get("authentication").is_none());
        assert!(value["spec"].get("clusterTags").is_none());
    }

    #[test]
    fn cce_create_request_serializes_cluster_tags() {
        let payload = CceCreateClusterRequest {
            kind: "Cluster".to_string(),
            api_version: "v3".to_string(),
            metadata: CceClusterCreateMetadata {
                name: "tagged-cluster".to_string(),
            },
            spec: CceClusterCreateSpec {
                cluster_type: "VirtualMachine".to_string(),
                flavor: "cce.s2.medium".to_string(),
                version: "v1.29".to_string(),
                description: None,
                host_network: CceHostNetwork {
                    vpc: "vpc-id".to_string(),
                    subnet: "subnet-id".to_string(),
                },
                container_network: CceContainerNetwork {
                    mode: "overlay_l2".to_string(),
                    cidr: "172.16.0.0/16".to_string(),
                },
                kubernetes_svc_ip_range: "10.247.0.0/16".to_string(),
                authentication: Some(CceAuthentication {
                    mode: "rbac".to_string(),
                }),
                cluster_tags: vec![CceClusterTag {
                    key: "env".to_string(),
                    value: "prod".to_string(),
                }],
            },
        };

        let value = serde_json::to_value(payload).expect("serialize cce create request with tags");
        assert_eq!(value["spec"]["clusterTags"][0]["key"], "env");
        assert_eq!(value["spec"]["clusterTags"][0]["value"], "prod");
    }

    #[test]
    fn cce_node_pool_list_response_deserializes_items() {
        let raw = r#"{
          "kind":"NodePoolList",
          "apiVersion":"v3",
          "items":[
            {
              "kind":"NodePool",
              "apiVersion":"v3",
              "metadata":{"id":"pool-1","name":"default-pool"},
              "spec":{"version":"v1.29"},
              "status":{"phase":"Running"}
            }
          ]
        }"#;

        let body: CceNodePoolListResponse =
            serde_json::from_str(raw).expect("deserialize cce node pool list response");
        assert_eq!(body.items.len(), 1);
        assert_eq!(body.kind.as_deref(), Some("NodePoolList"));
        assert_eq!(body.items[0].metadata["id"], "pool-1");
        assert_eq!(body.items[0].status["phase"], "Running");
    }

    #[test]
    fn cce_update_cluster_request_serializes_external_ip() {
        let payload = CceUpdateClusterRequest {
            spec: CceUpdateClusterSpec {
                cluster_external_ip: "1.2.3.4".to_string(),
            },
        };

        let value = serde_json::to_value(payload).expect("serialize cce cluster update payload");
        assert_eq!(value["spec"]["clusterExternalIP"], "1.2.3.4");
    }

    #[test]
    fn cce_cluster_cert_request_omits_context_when_none() {
        let payload = CceClusterCertRequest { context: None };
        let value = serde_json::to_value(payload).expect("serialize cce cluster cert request");
        assert!(value.get("context").is_none());
    }
}
