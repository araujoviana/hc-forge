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

#[derive(Debug, Clone, Serialize)]
pub struct CceCreateNodePoolRequest {
    pub kind: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub metadata: CceNodePoolCreateMetadata,
    pub spec: CceNodePoolCreateSpec,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolCreateMetadata {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolCreateSpec {
    #[serde(rename = "type")]
    pub node_pool_type: String,
    #[serde(rename = "initialNodeCount")]
    pub initial_node_count: u32,
    #[serde(rename = "nodeTemplate")]
    pub node_template: CceNodePoolTemplateSpec,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolTemplateSpec {
    pub flavor: String,
    pub az: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<CceNodePoolLogin>,
    #[serde(rename = "rootVolume")]
    pub root_volume: CceNodePoolVolume,
    #[serde(rename = "dataVolumes", skip_serializing_if = "Vec::is_empty", default)]
    pub data_volumes: Vec<CceNodePoolVolume>,
    #[serde(rename = "nodeNicSpec", skip_serializing_if = "Option::is_none")]
    pub node_nic_spec: Option<CceNodePoolNicSpec>,
    #[serde(rename = "billingMode")]
    pub billing_mode: u8,
    #[serde(rename = "extendParam", skip_serializing_if = "Option::is_none")]
    pub extend_param: Option<CceNodePoolExtendParam>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolLogin {
    #[serde(rename = "sshKey")]
    pub ssh_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolVolume {
    pub volumetype: String,
    pub size: u32,
    #[serde(rename = "extendParam", skip_serializing_if = "Option::is_none")]
    pub extend_param: Option<CceNodePoolVolumeExtendParam>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolVolumeExtendParam {
    #[serde(rename = "useType")]
    pub use_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolNicSpec {
    #[serde(rename = "primaryNic")]
    pub primary_nic: CceNodePoolPrimaryNic,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolPrimaryNic {
    #[serde(rename = "subnetId")]
    pub subnet_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CceNodePoolExtendParam {
    #[serde(rename = "maxPods", skip_serializing_if = "Option::is_none")]
    pub max_pods: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::{
        CceAuthentication, CceClusterCertRequest, CceClusterCreateMetadata, CceClusterCreateSpec,
        CceClusterTag, CceContainerNetwork, CceCreateClusterRequest, CceCreateNodePoolRequest,
        CceHostNetwork, CceNodePoolCreateMetadata, CceNodePoolCreateSpec, CceNodePoolExtendParam,
        CceNodePoolListResponse, CceNodePoolLogin, CceNodePoolNicSpec, CceNodePoolPrimaryNic,
        CceNodePoolTemplateSpec, CceNodePoolVolume, CceNodePoolVolumeExtendParam,
        CceUpdateClusterRequest, CceUpdateClusterSpec,
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

    #[test]
    fn cce_create_node_pool_request_serializes_expected_fields() {
        let payload = CceCreateNodePoolRequest {
            kind: "NodePool".to_string(),
            api_version: "v3".to_string(),
            metadata: CceNodePoolCreateMetadata {
                name: "workload-pool".to_string(),
            },
            spec: CceNodePoolCreateSpec {
                node_pool_type: "vm".to_string(),
                initial_node_count: 2,
                node_template: CceNodePoolTemplateSpec {
                    flavor: "c6.2xlarge.2".to_string(),
                    az: "sa-brazil-1a".to_string(),
                    os: Some("EulerOS 2.9".to_string()),
                    login: Some(CceNodePoolLogin {
                        ssh_key: "my-keypair".to_string(),
                    }),
                    root_volume: CceNodePoolVolume {
                        volumetype: "GPSSD".to_string(),
                        size: 40,
                        extend_param: None,
                    },
                    data_volumes: vec![CceNodePoolVolume {
                        volumetype: "GPSSD".to_string(),
                        size: 100,
                        extend_param: Some(CceNodePoolVolumeExtendParam {
                            use_type: "docker".to_string(),
                        }),
                    }],
                    node_nic_spec: Some(CceNodePoolNicSpec {
                        primary_nic: CceNodePoolPrimaryNic {
                            subnet_id: "subnet-id".to_string(),
                        },
                    }),
                    billing_mode: 0,
                    extend_param: Some(CceNodePoolExtendParam {
                        max_pods: Some(110),
                    }),
                },
            },
        };

        let value = serde_json::to_value(payload).expect("serialize cce create node pool request");
        assert_eq!(value["kind"], "NodePool");
        assert_eq!(value["apiVersion"], "v3");
        assert_eq!(value["metadata"]["name"], "workload-pool");
        assert_eq!(value["spec"]["type"], "vm");
        assert_eq!(value["spec"]["initialNodeCount"], 2);
        assert_eq!(value["spec"]["nodeTemplate"]["flavor"], "c6.2xlarge.2");
        assert_eq!(value["spec"]["nodeTemplate"]["az"], "sa-brazil-1a");
        assert_eq!(
            value["spec"]["nodeTemplate"]["login"]["sshKey"],
            "my-keypair"
        );
        assert_eq!(
            value["spec"]["nodeTemplate"]["rootVolume"]["volumetype"],
            "GPSSD"
        );
        assert_eq!(
            value["spec"]["nodeTemplate"]["dataVolumes"][0]["extendParam"]["useType"],
            "docker"
        );
        assert_eq!(
            value["spec"]["nodeTemplate"]["nodeNicSpec"]["primaryNic"]["subnetId"],
            "subnet-id"
        );
        assert_eq!(value["spec"]["nodeTemplate"]["billingMode"], 0);
        assert_eq!(value["spec"]["nodeTemplate"]["extendParam"]["maxPods"], 110);
    }

    #[test]
    fn cce_create_node_pool_request_omits_optional_fields_when_empty() {
        let payload = CceCreateNodePoolRequest {
            kind: "NodePool".to_string(),
            api_version: "v3".to_string(),
            metadata: CceNodePoolCreateMetadata {
                name: "minimal-pool".to_string(),
            },
            spec: CceNodePoolCreateSpec {
                node_pool_type: "vm".to_string(),
                initial_node_count: 1,
                node_template: CceNodePoolTemplateSpec {
                    flavor: "c6.large.2".to_string(),
                    az: "sa-brazil-1a".to_string(),
                    os: None,
                    login: None,
                    root_volume: CceNodePoolVolume {
                        volumetype: "SATA".to_string(),
                        size: 40,
                        extend_param: None,
                    },
                    data_volumes: Vec::new(),
                    node_nic_spec: None,
                    billing_mode: 0,
                    extend_param: None,
                },
            },
        };

        let value = serde_json::to_value(payload).expect("serialize minimal cce node pool request");
        assert!(value["spec"]["nodeTemplate"].get("os").is_none());
        assert!(value["spec"]["nodeTemplate"].get("login").is_none());
        assert!(value["spec"]["nodeTemplate"].get("dataVolumes").is_none());
        assert!(value["spec"]["nodeTemplate"].get("nodeNicSpec").is_none());
        assert!(value["spec"]["nodeTemplate"].get("extendParam").is_none());
    }
}
