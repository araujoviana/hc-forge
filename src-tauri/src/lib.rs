mod api;
mod validators;

use crate::api::models::cce::{CceClusterListResponse, CceNodePoolListResponse};
use crate::api::models::eip::EipListResponse;
use crate::api::models::evs::EvsListResponse;
use crate::api::models::ims::Image;
use crate::api::models::nat::NatGatewayListResponse;
use crate::api::models::obs::{ObsListBucketsResponse, ObsListObjectsResponse};
use crate::validators::{
    control_char_from_input, normalize_obs_bucket_name, normalize_obs_object_key,
    normalize_ssh_session_id,
};
use api::models::cce::{
    CceAuthentication, CceClusterCreateMetadata, CceClusterCreateSpec, CceClusterTag,
    CceContainerNetwork, CceCreateClusterRequest, CceHostNetwork,
};
use api::models::ecs::{
    Bandwidth, CreateEcsRequest, DataVolume, EcsListResponse, Eip, Flavor, Nic, PublicIp,
    RootVolume, Server,
};
use api::models::vpc::{Subnet, Vpc};
use api::{Credentials, CredentialsSource, HwcClient, ImageListFilters, ListParams};
use base64::Engine;
use chrono::Utc;
use log::{error, info, warn};
use rand::{distr::Alphanumeric, Rng};
use russh::{client, ChannelMsg, Disconnect};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;
use tokio::task::JoinHandle;

const RANDOM_NAME_PLACEHOLDER: &str = "ecs-<RANDOM-VALUE>";
const DEFAULT_EIP_TYPE: &str = "5_bgp";
const DEFAULT_BANDWIDTH_SHARE_TYPE: &str = "PER";
const DEFAULT_BANDWIDTH_CHARGE_MODE: &str = "traffic";
const DEFAULT_BANDWIDTH_SIZE: u32 = 100;
const MIN_BANDWIDTH_SIZE: u32 = 1;
const MAX_BANDWIDTH_SIZE: u32 = 300;
const OBS_BUCKET_NAME_MIN: usize = 3;
const OBS_BUCKET_NAME_MAX: usize = 63;
const OBS_PUT_OBJECT_MAX_BYTES: usize = 5 * 1024 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcsCreateParams {
    /// Server name or the literal "ecs-<RANDOM-VALUE>" to auto-generate.
    name: String,
    image_id: String,
    flavor_id: String,
    region: String,
    vpc_id: String,
    subnet_id: String,
    root_volume_type: String,
    root_volume_size: u32,
    eip: bool,
    eip_bandwidth_size: Option<u32>,
    admin_password: Option<String>,
    data_volumes: Option<Vec<EcsDataVolumeInput>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcsDataVolumeInput {
    volume_type: String,
    size: u32,
    count: Option<u32>,
    multiattach: Option<bool>,
    hw_passthrough: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcsDeleteParams {
    region: String,
    server_id: String,
    eip_id: Option<String>,
    delete_volume: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcsStopParams {
    region: String,
    server_id: String,
    stop_type: Option<String>,
}

/// AK/SK credentials input from the UI.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CredentialsInput {
    access_key: String,
    secret_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageFilters {
    visibility: Option<String>,
    image_type: Option<String>,
    flavor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListParamsInput {
    marker: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObsCreateBucketParams {
    region: String,
    bucket_name: String,
    default_storage_class: Option<String>,
    acl: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObsDeleteBucketParams {
    region: String,
    bucket_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObsListObjectsParams {
    region: String,
    bucket_name: String,
    prefix: Option<String>,
    marker: Option<String>,
    max_keys: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObsPutObjectParams {
    region: String,
    bucket_name: String,
    object_key: String,
    content_base64: String,
    content_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObsDeleteObjectParams {
    region: String,
    bucket_name: String,
    object_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObsGetObjectParams {
    region: String,
    bucket_name: String,
    object_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceCreateClusterParams {
    region: String,
    name: String,
    flavor: String,
    version: String,
    vpc_id: String,
    subnet_id: String,
    description: Option<String>,
    cluster_type: Option<String>,
    container_network_mode: Option<String>,
    container_network_cidr: Option<String>,
    kubernetes_svc_ip_range: Option<String>,
    authentication_mode: Option<String>,
    cluster_tag_env: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceDeleteClusterParams {
    region: String,
    cluster_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceListNodePoolsParams {
    region: String,
    cluster_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceGetJobParams {
    region: String,
    job_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceListNatGatewaysParams {
    region: String,
    vpc_id: String,
    subnet_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceCreateNatGatewayParams {
    region: String,
    name: String,
    vpc_id: String,
    subnet_id: String,
    description: Option<String>,
    spec: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceDeleteNatGatewayParams {
    region: String,
    nat_gateway_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceBindClusterApiEipParams {
    region: String,
    cluster_id: String,
    eip_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CceDownloadKubeconfigParams {
    region: String,
    cluster_id: String,
    context: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateEcsResult {
    status: String,
    status_code: u16,
    body: String,
}

#[derive(Debug, Serialize)]
struct DeleteOperationResult {
    status: String,
    status_code: Option<u16>,
    body: String,
}

#[derive(Debug, Serialize)]
struct DeleteEcsResult {
    ecs: DeleteOperationResult,
    eip: Option<DeleteOperationResult>,
}

#[derive(Debug, Serialize)]
struct StopEcsResult {
    ecs: DeleteOperationResult,
}

#[derive(Debug, Serialize)]
struct ObsOperationResult {
    status: String,
    status_code: u16,
    body: String,
}

#[derive(Debug, Serialize)]
struct ObsGetObjectResult {
    status: String,
    status_code: u16,
    content_base64: Option<String>,
    content_type: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Serialize)]
struct CceOperationResult {
    status: String,
    status_code: u16,
    body: String,
}

#[derive(Debug, Serialize)]
struct CceKubeconfigResult {
    status: String,
    status_code: u16,
    body: String,
    kubeconfig: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshConnectParams {
    session_id: String,
    host: String,
    port: Option<u16>,
    username: Option<String>,
    password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshExecParams {
    session_id: String,
    command: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshDisconnectParams {
    session_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshResizeParams {
    session_id: String,
    cols: u32,
    rows: u32,
    pixel_width: Option<u32>,
    pixel_height: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshSendControlParams {
    session_id: String,
    control: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshExecOneShotParams {
    session_id: String,
    host: String,
    port: Option<u16>,
    username: Option<String>,
    password: String,
    command: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshConnectResult {
    session_id: String,
    host: String,
    port: u16,
    username: String,
    connected_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshExecResult {
    session_id: String,
    command: String,
    stdout: String,
    stderr: String,
    exit_status: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshDisconnectResult {
    session_id: String,
    disconnected: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshResizeResult {
    session_id: String,
    cols: u32,
    rows: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshSendControlResult {
    session_id: String,
    control: String,
    sent: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshExecOneShotResult {
    session_id: String,
    host: String,
    port: u16,
    username: String,
    command: String,
    stdout: String,
    stderr: String,
    exit_status: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshStreamEvent {
    session_id: String,
    kind: String,
    text: String,
    at: String,
}

#[derive(Default)]
struct SshClientHandler;

impl client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

struct SshSessionEntry {
    handle: client::Handle<SshClientHandler>,
    shell_writer: russh::ChannelWriteHalf<client::Msg>,
    shell_reader_task: JoinHandle<()>,
    host: String,
    port: u16,
    username: String,
}

#[derive(Default)]
struct SshSessionStore {
    sessions: Mutex<HashMap<String, SshSessionEntry>>,
}

// Generate a ECS name when the placeholder is used.
fn normalize_server_name(input: &str) -> String {
    if input.trim().is_empty() || input == RANDOM_NAME_PLACEHOLDER {
        let ts = Utc::now().format("%Y%m%d-%H%M%S");
        let rand: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        return format!("ecs-{}-{}", ts, rand);
    }

    input.to_string()
}

// Require explicit credentials from the UI for every API call.
fn resolve_credentials(
    input: Option<CredentialsInput>,
) -> Result<(Credentials, CredentialsSource), String> {
    let input = input.ok_or_else(|| "Access Key and Secret Key are required.".to_string())?;
    let access_key = input.access_key.trim();
    let secret_key = input.secret_key.trim();

    if access_key.is_empty() || secret_key.is_empty() {
        return Err("Provide both Access Key and Secret Key.".to_string());
    }

    Ok((
        Credentials::new(access_key.to_string(), secret_key.to_string()),
        CredentialsSource::Explicit,
    ))
}

fn credentials_source_label(source: &CredentialsSource) -> String {
    match source {
        CredentialsSource::Explicit => "explicit".to_string(),
    }
}

fn operation_result(status: reqwest::StatusCode, body: String) -> DeleteOperationResult {
    DeleteOperationResult {
        status: status.to_string(),
        status_code: Some(status.as_u16()),
        body,
    }
}

fn operation_error_result(status: &str, body: String) -> DeleteOperationResult {
    DeleteOperationResult {
        status: status.to_string(),
        status_code: None,
        body,
    }
}

fn obs_operation_result(status: reqwest::StatusCode, body: String) -> ObsOperationResult {
    ObsOperationResult {
        status: status.to_string(),
        status_code: status.as_u16(),
        body,
    }
}

fn cce_operation_result(status: reqwest::StatusCode, body: String) -> CceOperationResult {
    CceOperationResult {
        status: status.to_string(),
        status_code: status.as_u16(),
        body,
    }
}

fn parse_json_or_string(raw: &str) -> Value {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Value::String(String::new())
    } else {
        serde_json::from_str(trimmed).unwrap_or_else(|_| Value::String(trimmed.to_string()))
    }
}

fn extract_nat_gateway_id(raw_body: &str) -> Option<String> {
    let payload: Value = serde_json::from_str(raw_body).ok()?;
    payload
        .get("nat_gateway")
        .and_then(|item| item.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn extract_eip_id_and_address(raw_body: &str) -> (Option<String>, Option<String>) {
    let payload: Value = match serde_json::from_str(raw_body) {
        Ok(value) => value,
        Err(_) => return (None, None),
    };
    let publicip = payload.get("publicip");
    let id = publicip
        .and_then(|item| item.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let address = publicip
        .and_then(|item| {
            item.get("public_ip_address")
                .or_else(|| item.get("public_ip"))
        })
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    (id, address)
}

fn extract_cluster_kubeconfig(raw_body: &str) -> Option<String> {
    let trimmed = raw_body.trim();
    if trimmed.is_empty() {
        return None;
    }
    let payload: Value = match serde_json::from_str(trimmed) {
        Ok(value) => value,
        Err(_) => return Some(trimmed.to_string()),
    };
    if let Some(value) = payload
        .get("kubeconfig")
        .or_else(|| payload.get("kube_config"))
        .or_else(|| payload.get("kubeConfig"))
        .or_else(|| payload.get("config"))
        .and_then(Value::as_str)
    {
        let text = value.trim();
        if !text.is_empty() {
            return Some(text.to_string());
        }
    }
    if let Some(certs) = payload.get("certs") {
        if let Some(value) = certs
            .get("kubeconfig")
            .or_else(|| certs.get("kube_config"))
            .or_else(|| certs.get("kubeConfig"))
            .or_else(|| certs.get("config"))
            .and_then(Value::as_str)
        {
            let text = value.trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    if payload.get("clusters").is_some()
        && payload.get("contexts").is_some()
        && payload.get("users").is_some()
    {
        return serde_json::to_string_pretty(&payload).ok();
    }
    None
}

/// List VPCs for the given region so the UI can populate a dropdown.
#[tauri::command]
async fn list_vpcs(
    region: String,
    credentials: Option<CredentialsInput>,
) -> Result<Vec<Vpc>, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!("Listing VPCs: source={} region={}", source_label, region);

    let client = HwcClient::new(credentials);
    client.list_vpcs(&region).await.map_err(|err| {
        error!("Failed to list VPCs: region={} error={}", region, err);
        err.to_string()
    })
}

/// List subnets for the selected VPC so the UI can populate a dropdown.
#[tauri::command]
async fn list_subnets(
    region: String,
    vpc_id: String,
    credentials: Option<CredentialsInput>,
) -> Result<Vec<Subnet>, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!(
        "Listing subnets: source={} region={} vpc_id={}",
        source_label, region, vpc_id
    );

    let client = HwcClient::new(credentials);
    client.list_subnets(&region, &vpc_id).await.map_err(|err| {
        error!(
            "Failed to list subnets: region={} vpc_id={} error={}",
            region, vpc_id, err
        );
        err.to_string()
    })
}

/// List images for the given region so the UI can populate a dropdown.
#[tauri::command]
async fn list_images(
    region: String,
    filters: Option<ImageFilters>,
    credentials: Option<CredentialsInput>,
) -> Result<Vec<Image>, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!("Listing images: source={} region={}", source_label, region);

    let client = HwcClient::new(credentials);
    let filters = filters.map(|input| ImageListFilters {
        visibility: input.visibility,
        image_type: input.image_type,
        flavor_id: input.flavor_id,
    });
    let images = client.list_images(&region, filters).await.map_err(|err| {
        error!("Failed to list images: region={} error={}", region, err);
        err.to_string()
    })?;

    warn!("Raw images response: {:#?}", images);
    info!("Found {} images in region {}", images.len(), region);

    Ok(images)
}

/// List flavors for the given region so the UI can populate a dropdown.
#[tauri::command]
async fn list_flavors(
    region: String,
    credentials: Option<CredentialsInput>,
) -> Result<Vec<Flavor>, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!("Listing flavors: source={} region={}", source_label, region);

    let client = HwcClient::new(credentials);
    client.list_flavors(&region).await.map_err(|err| {
        error!("Failed to list flavors: region={} error={}", region, err);
        err.to_string()
    })
}

/// List elastic IPs for the given region.
#[tauri::command]
async fn list_eips(
    region: String,
    params: Option<ListParamsInput>,
    credentials: Option<CredentialsInput>,
) -> Result<EipListResponse, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!("Listing EIPs: source={} region={}", source_label, region);

    let client = HwcClient::new(credentials);
    let params = params.map(|input| ListParams {
        marker: input.marker,
        limit: input.limit,
        offset: input.offset,
    });

    client.list_eips(&region, params).await.map_err(|err| {
        error!("Failed to list EIPs: region={} error={:#}", region, err);
        err.to_string()
    })
}

/// List ECS instances for the given region.
#[tauri::command]
async fn list_ecses(
    region: String,
    params: Option<ListParamsInput>,
    credentials: Option<CredentialsInput>,
) -> Result<EcsListResponse, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!(
        "Listing ECS instances: source={} region={}",
        source_label, region
    );

    let client = HwcClient::new(credentials);
    let params = params.map(|input| ListParams {
        marker: input.marker,
        limit: input.limit,
        offset: input.offset,
    });

    client.list_ecses(&region, params).await.map_err(|err| {
        error!(
            "Failed to list ECS instances: region={} error={}",
            region, err
        );
        err.to_string()
    })
}

/// List EVS disks for the given region.
#[tauri::command]
async fn list_evss(
    region: String,
    params: Option<ListParamsInput>,
    credentials: Option<CredentialsInput>,
) -> Result<EvsListResponse, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!(
        "Listing EVS disks: source={} region={}",
        source_label, region
    );

    let client = HwcClient::new(credentials);
    let params = params.map(|input| ListParams {
        marker: input.marker,
        limit: input.limit,
        offset: input.offset,
    });

    client.list_evss(&region, params).await.map_err(|err| {
        error!("Failed to list EVS disks: region={} error={}", region, err);
        err.to_string()
    })
}

/// List CCE clusters for the selected region.
#[tauri::command]
async fn list_cce_clusters(
    region: String,
    credentials: Option<CredentialsInput>,
) -> Result<CceClusterListResponse, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!(
        "Listing CCE clusters: source={} region={}",
        source_label, region
    );

    let client = HwcClient::new(credentials);
    client.list_cce_clusters(&region).await.map_err(|err| {
        error!(
            "Failed to list CCE clusters: region={} error={}",
            region, err
        );
        err.to_string()
    })
}

/// Create one CCE cluster.
#[tauri::command]
async fn create_cce_cluster(
    params: CceCreateClusterParams,
    credentials: Option<CredentialsInput>,
) -> Result<CceOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let cluster_name = params.name.trim();
    if cluster_name.is_empty() {
        return Err("CCE cluster name is required.".to_string());
    }
    let flavor = params.flavor.trim();
    if flavor.is_empty() {
        return Err("CCE cluster flavor is required.".to_string());
    }
    let version = params.version.trim();
    if version.is_empty() {
        return Err("CCE Kubernetes version is required.".to_string());
    }
    let vpc_id = params.vpc_id.trim();
    if vpc_id.is_empty() {
        return Err("CCE VPC is required.".to_string());
    }
    let subnet_id = params.subnet_id.trim();
    if subnet_id.is_empty() {
        return Err("CCE subnet is required.".to_string());
    }

    let cluster_type = params
        .cluster_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("VirtualMachine")
        .to_string();
    let container_network_mode = params
        .container_network_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("overlay_l2")
        .to_string();
    let container_network_cidr = params
        .container_network_cidr
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("172.16.0.0/16")
        .to_string();
    let kubernetes_svc_ip_range = params
        .kubernetes_svc_ip_range
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("10.247.0.0/16")
        .to_string();
    let authentication_mode = params
        .authentication_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("rbac")
        .to_string();

    let mut cluster_tags = Vec::new();
    if let Some(env) = params
        .cluster_tag_env
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        cluster_tags.push(CceClusterTag {
            key: "env".to_string(),
            value: env.to_string(),
        });
    }

    let body = CceCreateClusterRequest {
        kind: "Cluster".to_string(),
        api_version: "v3".to_string(),
        metadata: CceClusterCreateMetadata {
            name: cluster_name.to_string(),
        },
        spec: CceClusterCreateSpec {
            cluster_type,
            flavor: flavor.to_string(),
            version: version.to_string(),
            description: params
                .description
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            host_network: CceHostNetwork {
                vpc: vpc_id.to_string(),
                subnet: subnet_id.to_string(),
            },
            container_network: CceContainerNetwork {
                mode: container_network_mode,
                cidr: container_network_cidr,
            },
            kubernetes_svc_ip_range,
            authentication: Some(CceAuthentication {
                mode: authentication_mode,
            }),
            cluster_tags,
        },
    };

    let source_label = credentials_source_label(&source);
    info!(
        "Creating CCE cluster: source={} region={} name={} flavor={} version={} vpc_id={} subnet_id={}",
        source_label, params.region, cluster_name, flavor, version, vpc_id, subnet_id
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .create_cce_cluster(&params.region, &body)
        .await
        .map_err(|err| {
            error!(
                "Failed to create CCE cluster: region={} name={} error={}",
                params.region, cluster_name, err
            );
            err.to_string()
        })?;

    Ok(cce_operation_result(status, body))
}

/// Delete one CCE cluster.
#[tauri::command]
async fn delete_cce_cluster(
    params: CceDeleteClusterParams,
    credentials: Option<CredentialsInput>,
) -> Result<CceOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let cluster_id = params.cluster_id.trim();
    if cluster_id.is_empty() {
        return Err("CCE cluster ID is required.".to_string());
    }

    let source_label = credentials_source_label(&source);
    info!(
        "Deleting CCE cluster: source={} region={} cluster_id={}",
        source_label, params.region, cluster_id
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .delete_cce_cluster(&params.region, cluster_id)
        .await
        .map_err(|err| {
            error!(
                "Failed to delete CCE cluster: region={} cluster_id={} error={}",
                params.region, cluster_id, err
            );
            err.to_string()
        })?;

    Ok(cce_operation_result(status, body))
}

/// List node pools for one CCE cluster.
#[tauri::command]
async fn list_cce_node_pools(
    params: CceListNodePoolsParams,
    credentials: Option<CredentialsInput>,
) -> Result<CceNodePoolListResponse, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let cluster_id = params.cluster_id.trim();
    if cluster_id.is_empty() {
        return Err("CCE cluster ID is required.".to_string());
    }

    let source_label = credentials_source_label(&source);
    info!(
        "Listing CCE node pools: source={} region={} cluster_id={}",
        source_label, params.region, cluster_id
    );

    let client = HwcClient::new(credentials);
    client
        .list_cce_node_pools(&params.region, cluster_id)
        .await
        .map_err(|err| {
            error!(
                "Failed to list CCE node pools: region={} cluster_id={} error={}",
                params.region, cluster_id, err
            );
            err.to_string()
        })
}

/// Query one CCE job status.
#[tauri::command]
async fn get_cce_job(
    params: CceGetJobParams,
    credentials: Option<CredentialsInput>,
) -> Result<CceOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let job_id = params.job_id.trim();
    if job_id.is_empty() {
        return Err("CCE job ID is required.".to_string());
    }

    let source_label = credentials_source_label(&source);
    info!(
        "Querying CCE job: source={} region={} job_id={}",
        source_label, params.region, job_id
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .get_cce_job(&params.region, job_id)
        .await
        .map_err(|err| {
            error!(
                "Failed to query CCE job: region={} job_id={} error={}",
                params.region, job_id, err
            );
            err.to_string()
        })?;

    Ok(cce_operation_result(status, body))
}

/// List NAT gateways scoped to the selected CCE VPC/subnet.
#[tauri::command]
async fn list_cce_nat_gateways(
    params: CceListNatGatewaysParams,
    credentials: Option<CredentialsInput>,
) -> Result<NatGatewayListResponse, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let vpc_id = params.vpc_id.trim();
    if vpc_id.is_empty() {
        return Err("CCE NAT requires a VPC.".to_string());
    }
    let subnet_id = params.subnet_id.trim();
    if subnet_id.is_empty() {
        return Err("CCE NAT requires a subnet.".to_string());
    }

    let source_label = credentials_source_label(&source);
    info!(
        "Listing CCE NAT gateways: source={} region={} vpc_id={} subnet_id={}",
        source_label, params.region, vpc_id, subnet_id
    );

    let client = HwcClient::new(credentials);
    client
        .list_nat_gateways(&params.region, Some(vpc_id), Some(subnet_id))
        .await
        .map_err(|err| {
            error!(
                "Failed to list CCE NAT gateways: region={} vpc_id={} subnet_id={} error={}",
                params.region, vpc_id, subnet_id, err
            );
            err.to_string()
        })
}

/// Create one NAT gateway for the selected CCE VPC/subnet.
#[tauri::command]
async fn create_cce_nat_gateway(
    params: CceCreateNatGatewayParams,
    credentials: Option<CredentialsInput>,
) -> Result<CceOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let name = params.name.trim();
    if name.is_empty() {
        return Err("CCE NAT gateway name is required.".to_string());
    }
    let vpc_id = params.vpc_id.trim();
    if vpc_id.is_empty() {
        return Err("CCE NAT requires a VPC.".to_string());
    }
    let subnet_id = params.subnet_id.trim();
    if subnet_id.is_empty() {
        return Err("CCE NAT requires a subnet.".to_string());
    }
    let spec = params
        .spec
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("1");
    if spec != "1" {
        return Err("Unsupported NAT gateway spec. Use spec 1.".to_string());
    }

    let description = params
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let source_label = credentials_source_label(&source);
    info!(
        "Creating CCE NAT gateway with EIP+SNAT bootstrap: source={} region={} name={} vpc_id={} subnet_id={} spec={}",
        source_label, params.region, name, vpc_id, subnet_id, spec
    );

    let client = HwcClient::new(credentials);
    let (nat_status, nat_body) = client
        .create_nat_gateway(&params.region, name, description, spec, vpc_id, subnet_id)
        .await
        .map_err(|err| {
            error!(
                "Failed to create CCE NAT gateway: region={} name={} error={}",
                params.region, name, err
            );
            err.to_string()
        })?;
    let mut summary = json!({
        "requested": {
            "region": params.region,
            "name": name,
            "vpc_id": vpc_id,
            "subnet_id": subnet_id,
            "spec": spec,
            "auto_bind_eip": true,
            "auto_create_snat": true
        },
        "nat_gateway": {
            "status": nat_status.to_string(),
            "status_code": nat_status.as_u16(),
            "body": parse_json_or_string(&nat_body)
        }
    });

    if !nat_status.is_success() {
        let body = serde_json::to_string_pretty(&summary).unwrap_or_else(|_| summary.to_string());
        return Ok(cce_operation_result(nat_status, body));
    }

    let nat_gateway_id = match extract_nat_gateway_id(&nat_body) {
        Some(value) => value,
        None => {
            summary["error"] =
                json!("NAT gateway create succeeded but response did not contain nat_gateway.id.");
            let body =
                serde_json::to_string_pretty(&summary).unwrap_or_else(|_| summary.to_string());
            return Ok(cce_operation_result(
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                body,
            ));
        }
    };
    summary["nat_gateway"]["id"] = json!(nat_gateway_id.clone());

    let mut last_nat_status = String::new();
    for attempt in 1..=8 {
        match client
            .get_nat_gateway(&params.region, &nat_gateway_id)
            .await
        {
            Ok(response) => {
                let status_text = response
                    .nat_gateway
                    .status
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or("");
                if !status_text.is_empty() {
                    last_nat_status = status_text.to_string();
                }
                if status_text.eq_ignore_ascii_case("ACTIVE") {
                    summary["nat_gateway"]["ready_status"] = json!(status_text);
                    summary["nat_gateway"]["ready_attempt"] = json!(attempt);
                    break;
                }
            }
            Err(err) => {
                warn!(
                    "Failed to poll NAT gateway status after create: region={} nat_gateway_id={} error={}",
                    params.region, nat_gateway_id, err
                );
            }
        }
        if attempt < 8 {
            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    }
    if !last_nat_status.is_empty() {
        summary["nat_gateway"]["last_observed_status"] = json!(last_nat_status);
    }

    let eip_name = format!("{}-eip", name);
    let (eip_status, eip_body) = client
        .create_eip(&params.region, DEFAULT_BANDWIDTH_SIZE, Some(&eip_name))
        .await
        .map_err(|err| {
            error!(
                "Failed to create EIP for CCE NAT bootstrap: region={} nat_gateway_id={} error={}",
                params.region, nat_gateway_id, err
            );
            err.to_string()
        })?;
    summary["eip"] = json!({
        "status": eip_status.to_string(),
        "status_code": eip_status.as_u16(),
        "body": parse_json_or_string(&eip_body)
    });

    if !eip_status.is_success() {
        let body = serde_json::to_string_pretty(&summary).unwrap_or_else(|_| summary.to_string());
        return Ok(cce_operation_result(eip_status, body));
    }

    let (eip_id, eip_address) = extract_eip_id_and_address(&eip_body);
    let eip_id = match eip_id {
        Some(value) => value,
        None => {
            summary["error"] =
                json!("EIP create succeeded but response did not contain publicip.id.");
            let body =
                serde_json::to_string_pretty(&summary).unwrap_or_else(|_| summary.to_string());
            return Ok(cce_operation_result(
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                body,
            ));
        }
    };
    summary["eip"]["id"] = json!(eip_id.clone());
    if let Some(address) = eip_address {
        summary["eip"]["address"] = json!(address);
    }

    let (snat_status, snat_body) = client
        .create_snat_rule(&params.region, &nat_gateway_id, subnet_id, &eip_id)
        .await
        .map_err(|err| {
            error!(
                "Failed to create SNAT rule for CCE NAT bootstrap: region={} nat_gateway_id={} eip_id={} error={}",
                params.region, nat_gateway_id, eip_id, err
            );
            err.to_string()
        })?;
    summary["snat_rule"] = json!({
        "status": snat_status.to_string(),
        "status_code": snat_status.as_u16(),
        "body": parse_json_or_string(&snat_body)
    });

    let body = serde_json::to_string_pretty(&summary).unwrap_or_else(|_| summary.to_string());
    Ok(cce_operation_result(snat_status, body))
}

/// Delete one NAT gateway by ID.
#[tauri::command]
async fn delete_cce_nat_gateway(
    params: CceDeleteNatGatewayParams,
    credentials: Option<CredentialsInput>,
) -> Result<CceOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let nat_gateway_id = params.nat_gateway_id.trim();
    if nat_gateway_id.is_empty() {
        return Err("CCE NAT gateway ID is required.".to_string());
    }

    let source_label = credentials_source_label(&source);
    info!(
        "Deleting CCE NAT gateway: source={} region={} nat_gateway_id={}",
        source_label, params.region, nat_gateway_id
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .delete_nat_gateway(&params.region, nat_gateway_id)
        .await
        .map_err(|err| {
            error!(
                "Failed to delete CCE NAT gateway: region={} nat_gateway_id={} error={}",
                params.region, nat_gateway_id, err
            );
            err.to_string()
        })?;

    Ok(cce_operation_result(status, body))
}

/// Bind a public EIP to one CCE cluster API endpoint for remote kubeconfig access.
#[tauri::command]
async fn bind_cce_cluster_api_eip(
    params: CceBindClusterApiEipParams,
    credentials: Option<CredentialsInput>,
) -> Result<CceOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let cluster_id = params.cluster_id.trim();
    if cluster_id.is_empty() {
        return Err("CCE cluster ID is required.".to_string());
    }
    let eip_address = params.eip_address.trim();
    if eip_address.is_empty() {
        return Err("CCE API EIP address is required.".to_string());
    }

    let source_label = credentials_source_label(&source);
    info!(
        "Binding CCE cluster API EIP: source={} region={} cluster_id={} eip_address={}",
        source_label, params.region, cluster_id, eip_address
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .update_cce_cluster_external_ip(&params.region, cluster_id, eip_address)
        .await
        .map_err(|err| {
            error!(
                "Failed to bind CCE cluster API EIP: region={} cluster_id={} eip_address={} error={}",
                params.region, cluster_id, eip_address, err
            );
            err.to_string()
        })?;

    Ok(cce_operation_result(status, body))
}

/// Request cluster kubeconfig payload (clustercert API) for local kubectl access.
#[tauri::command]
async fn get_cce_cluster_kubeconfig(
    params: CceDownloadKubeconfigParams,
    credentials: Option<CredentialsInput>,
) -> Result<CceKubeconfigResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let cluster_id = params.cluster_id.trim();
    if cluster_id.is_empty() {
        return Err("CCE cluster ID is required.".to_string());
    }

    let context = params
        .context
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("external");

    let source_label = credentials_source_label(&source);
    info!(
        "Requesting CCE cluster kubeconfig: source={} region={} cluster_id={} context={}",
        source_label, params.region, cluster_id, context
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .get_cce_cluster_kubeconfig(&params.region, cluster_id, Some(context))
        .await
        .map_err(|err| {
            error!(
                "Failed to request CCE cluster kubeconfig: region={} cluster_id={} error={}",
                params.region, cluster_id, err
            );
            err.to_string()
        })?;
    let kubeconfig = if status.is_success() {
        extract_cluster_kubeconfig(&body)
    } else {
        None
    };

    Ok(CceKubeconfigResult {
        status: status.to_string(),
        status_code: status.as_u16(),
        body,
        kubeconfig,
    })
}

/// List OBS buckets for the selected region.
#[tauri::command]
async fn list_obs_buckets(
    region: String,
    credentials: Option<CredentialsInput>,
) -> Result<ObsListBucketsResponse, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!(
        "Listing OBS buckets: source={} region={}",
        source_label, region
    );

    let client = HwcClient::new(credentials);
    client.list_obs_buckets(&region).await.map_err(|err| {
        error!(
            "Failed to list OBS buckets: region={} error={}",
            region, err
        );
        err.to_string()
    })
}

/// Create one OBS bucket.
#[tauri::command]
async fn create_obs_bucket(
    params: ObsCreateBucketParams,
    credentials: Option<CredentialsInput>,
) -> Result<ObsOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let bucket_name = normalize_obs_bucket_name(
        &params.bucket_name,
        OBS_BUCKET_NAME_MIN,
        OBS_BUCKET_NAME_MAX,
    )?;
    let source_label = credentials_source_label(&source);
    info!(
        "Creating OBS bucket: source={} region={} bucket={}",
        source_label, params.region, bucket_name
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .create_obs_bucket(
            &params.region,
            &bucket_name,
            params.default_storage_class.as_deref(),
            params.acl.as_deref(),
        )
        .await
        .map_err(|err| {
            error!(
                "Failed to create OBS bucket: region={} bucket={} error={}",
                params.region, bucket_name, err
            );
            err.to_string()
        })?;

    Ok(obs_operation_result(status, body))
}

/// Delete one OBS bucket.
#[tauri::command]
async fn delete_obs_bucket(
    params: ObsDeleteBucketParams,
    credentials: Option<CredentialsInput>,
) -> Result<ObsOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let bucket_name = normalize_obs_bucket_name(
        &params.bucket_name,
        OBS_BUCKET_NAME_MIN,
        OBS_BUCKET_NAME_MAX,
    )?;
    let source_label = credentials_source_label(&source);
    info!(
        "Deleting OBS bucket: source={} region={} bucket={}",
        source_label, params.region, bucket_name
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .delete_obs_bucket(&params.region, &bucket_name)
        .await
        .map_err(|err| {
            error!(
                "Failed to delete OBS bucket: region={} bucket={} error={}",
                params.region, bucket_name, err
            );
            err.to_string()
        })?;

    Ok(obs_operation_result(status, body))
}

/// List objects for one OBS bucket.
#[tauri::command]
async fn list_obs_objects(
    params: ObsListObjectsParams,
    credentials: Option<CredentialsInput>,
) -> Result<ObsListObjectsResponse, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let bucket_name = normalize_obs_bucket_name(
        &params.bucket_name,
        OBS_BUCKET_NAME_MIN,
        OBS_BUCKET_NAME_MAX,
    )?;
    let source_label = credentials_source_label(&source);
    info!(
        "Listing OBS objects: source={} region={} bucket={}",
        source_label, params.region, bucket_name
    );

    let client = HwcClient::new(credentials);
    client
        .list_obs_objects(
            &params.region,
            &bucket_name,
            params.prefix.as_deref(),
            params.marker.as_deref(),
            params.max_keys,
        )
        .await
        .map_err(|err| {
            error!(
                "Failed to list OBS objects: region={} bucket={} error={}",
                params.region, bucket_name, err
            );
            err.to_string()
        })
}

/// Upload one object to OBS.
#[tauri::command]
async fn put_obs_object(
    params: ObsPutObjectParams,
    credentials: Option<CredentialsInput>,
) -> Result<ObsOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let bucket_name = normalize_obs_bucket_name(
        &params.bucket_name,
        OBS_BUCKET_NAME_MIN,
        OBS_BUCKET_NAME_MAX,
    )?;
    let object_key = normalize_obs_object_key(&params.object_key)?;
    let source_label = credentials_source_label(&source);
    info!(
        "Uploading OBS object: source={} region={} bucket={} key={}",
        source_label, params.region, bucket_name, object_key
    );

    let content = base64::engine::general_purpose::STANDARD
        .decode(params.content_base64.trim())
        .map_err(|err| format!("Failed to decode base64 object payload: {}", err))?;
    if content.is_empty() {
        return Err("OBS upload payload is empty.".to_string());
    }
    if content.len() > OBS_PUT_OBJECT_MAX_BYTES {
        return Err(format!(
            "OBS PutObject supports up to {} bytes (5 GB). Use multipart upload for larger files.",
            OBS_PUT_OBJECT_MAX_BYTES
        ));
    }

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .put_obs_object(
            &params.region,
            &bucket_name,
            &object_key,
            content,
            params.content_type.as_deref(),
        )
        .await
        .map_err(|err| {
            error!(
                "Failed to upload OBS object: region={} bucket={} key={} error={}",
                params.region, bucket_name, object_key, err
            );
            err.to_string()
        })?;

    Ok(obs_operation_result(status, body))
}

/// Download one object from OBS.
#[tauri::command]
async fn get_obs_object(
    params: ObsGetObjectParams,
    credentials: Option<CredentialsInput>,
) -> Result<ObsGetObjectResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let bucket_name = normalize_obs_bucket_name(
        &params.bucket_name,
        OBS_BUCKET_NAME_MIN,
        OBS_BUCKET_NAME_MAX,
    )?;
    let object_key = normalize_obs_object_key(&params.object_key)?;
    let source_label = credentials_source_label(&source);
    info!(
        "Downloading OBS object: source={} region={} bucket={} key={}",
        source_label, params.region, bucket_name, object_key
    );

    let client = HwcClient::new(credentials);
    let (status, content, content_type) = client
        .get_obs_object(&params.region, &bucket_name, &object_key)
        .await
        .map_err(|err| {
            error!(
                "Failed to download OBS object: region={} bucket={} key={} error={}",
                params.region, bucket_name, object_key, err
            );
            err.to_string()
        })?;

    let status_code = status.as_u16();
    let success = status.is_success();
    let body = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&content).into_owned())
    };

    Ok(ObsGetObjectResult {
        status: status.to_string(),
        status_code,
        content_base64: if success {
            Some(base64::engine::general_purpose::STANDARD.encode(content))
        } else {
            None
        },
        content_type,
        body,
    })
}

/// Delete one object from OBS.
#[tauri::command]
async fn delete_obs_object(
    params: ObsDeleteObjectParams,
    credentials: Option<CredentialsInput>,
) -> Result<ObsOperationResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let bucket_name = normalize_obs_bucket_name(
        &params.bucket_name,
        OBS_BUCKET_NAME_MIN,
        OBS_BUCKET_NAME_MAX,
    )?;
    let object_key = normalize_obs_object_key(&params.object_key)?;
    let source_label = credentials_source_label(&source);
    info!(
        "Deleting OBS object: source={} region={} bucket={} key={}",
        source_label, params.region, bucket_name, object_key
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .delete_obs_object(&params.region, &bucket_name, &object_key)
        .await
        .map_err(|err| {
            error!(
                "Failed to delete OBS object: region={} bucket={} key={} error={}",
                params.region, bucket_name, object_key, err
            );
            err.to_string()
        })?;

    Ok(obs_operation_result(status, body))
}

/// Create an ECS instance using the same core flow as the old CLI.
#[tauri::command]
async fn create_ecs(
    params: EcsCreateParams,
    credentials: Option<CredentialsInput>,
) -> Result<CreateEcsResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!(
        "Creating ECS instance: source={} region={} vpc_id={} subnet_id={} allocate_eip={}",
        source_label, params.region, params.vpc_id, params.subnet_id, params.eip
    );

    let server_name = normalize_server_name(&params.name);
    let admin_password = params
        .admin_password
        .as_deref()
        .map(str::trim)
        .filter(|password| !password.is_empty())
        .map(|password| password.to_string());

    let eip_bandwidth_size = params.eip_bandwidth_size.unwrap_or(DEFAULT_BANDWIDTH_SIZE);
    if params.eip && !(MIN_BANDWIDTH_SIZE..=MAX_BANDWIDTH_SIZE).contains(&eip_bandwidth_size) {
        return Err(format!(
            "EIP bandwidth size must be between {} and {} Mbit/s for charge_mode=traffic.",
            MIN_BANDWIDTH_SIZE, MAX_BANDWIDTH_SIZE
        ));
    }

    let data_volumes = params
        .data_volumes
        .unwrap_or_default()
        .into_iter()
        .map(|volume| {
            let volume_type = volume.volume_type.trim().to_string();
            if volume_type.is_empty() {
                return Err("Data disk volume type is required.".to_string());
            }
            if volume.size == 0 {
                return Err("Data disk size must be greater than 0 GB.".to_string());
            }
            let count = volume.count.unwrap_or(1);
            if count == 0 {
                return Err("Data disk count must be at least 1.".to_string());
            }
            Ok(DataVolume {
                volumetype: volume_type,
                size: volume.size,
                count: Some(count),
                multiattach: volume.multiattach,
                hw_passthrough: volume.hw_passthrough,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let publicip = if params.eip {
        Some(PublicIp {
            eip: Eip {
                ip_type: DEFAULT_EIP_TYPE.into(),
                bandwidth: Bandwidth {
                    size: eip_bandwidth_size,
                    share_type: DEFAULT_BANDWIDTH_SHARE_TYPE.into(),
                    charge_mode: DEFAULT_BANDWIDTH_CHARGE_MODE.into(),
                },
            },
        })
    } else {
        None
    };

    let body = CreateEcsRequest {
        server: Server {
            name: server_name,
            image_ref: params.image_id,
            flavor_ref: params.flavor_id,
            vpcid: params.vpc_id,
            nics: vec![Nic {
                subnet_id: params.subnet_id,
            }],
            root_volume: RootVolume {
                volumetype: params.root_volume_type,
                size: params.root_volume_size,
            },
            data_volumes,
            publicip,
            admin_pass: admin_password,
        },
    };

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .create_ecs(&params.region, &body)
        .await
        .map_err(|err| {
            error!(
                "Failed to create ECS: region={} error={}",
                params.region, err
            );
            err.to_string()
        })?;

    Ok(CreateEcsResult {
        status: status.to_string(),
        status_code: status.as_u16(),
        body,
    })
}

/// Delete an ECS instance and, when possible, its attached EIP.
#[tauri::command]
async fn delete_ecs_with_eip(
    params: EcsDeleteParams,
    credentials: Option<CredentialsInput>,
) -> Result<DeleteEcsResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    info!(
        "Deleting ECS instance: source={} region={} server_id={}",
        source_label, params.region, params.server_id
    );

    let delete_volume = params.delete_volume.unwrap_or(true);
    let client = HwcClient::new(credentials);
    let (ecs_status, ecs_body) = client
        .delete_ecs(&params.region, &params.server_id, true, delete_volume)
        .await
        .map_err(|err| {
            error!(
                "Failed to delete ECS: region={} server_id={} error={}",
                params.region, params.server_id, err
            );
            err.to_string()
        })?;

    let ecs_result = operation_result(ecs_status, ecs_body);
    let ecs_success = ecs_result
        .status_code
        .is_some_and(|code| (200..300).contains(&code));
    let eip_id = params
        .eip_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(|id| id.to_string());

    let eip_result = if !ecs_success {
        eip_id.map(|id| {
            operation_error_result(
                "skipped",
                format!(
                    "Skipped EIP deletion for {} because ECS deletion did not return success.",
                    id
                ),
            )
        })
    } else if let Some(eip_id) = eip_id {
        match client.delete_eip(&params.region, &eip_id).await {
            Ok((status, body)) => Some(operation_result(status, body)),
            Err(err) => {
                warn!(
                    "Failed to delete EIP after ECS delete: region={} eip_id={} error={}",
                    params.region, eip_id, err
                );
                Some(operation_error_result("error", err.to_string()))
            }
        }
    } else {
        None
    };

    Ok(DeleteEcsResult {
        ecs: ecs_result,
        eip: eip_result,
    })
}

/// Stop one ECS instance using SOFT or HARD stop type.
#[tauri::command]
async fn stop_ecs(
    params: EcsStopParams,
    credentials: Option<CredentialsInput>,
) -> Result<StopEcsResult, String> {
    let (credentials, source) = resolve_credentials(credentials).map_err(|err| {
        error!("Failed to resolve credentials: {}", err);
        err
    })?;

    let source_label = credentials_source_label(&source);
    let requested_type = params
        .stop_type
        .as_deref()
        .map(str::trim)
        .filter(|kind| !kind.is_empty())
        .unwrap_or("SOFT")
        .to_ascii_uppercase();
    let stop_type = if requested_type == "HARD" {
        "HARD"
    } else {
        "SOFT"
    };

    info!(
        "Stopping ECS instance: source={} region={} server_id={} type={}",
        source_label, params.region, params.server_id, stop_type
    );

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .stop_ecs(&params.region, &params.server_id, stop_type)
        .await
        .map_err(|err| {
            error!(
                "Failed to stop ECS: region={} server_id={} error={}",
                params.region, params.server_id, err
            );
            err.to_string()
        })?;

    Ok(StopEcsResult {
        ecs: operation_result(status, body),
    })
}

fn lock_ssh_sessions<'a>(
    state: &'a tauri::State<'_, SshSessionStore>,
) -> Result<std::sync::MutexGuard<'a, HashMap<String, SshSessionEntry>>, String> {
    state
        .sessions
        .lock()
        .map_err(|_| "SSH session store is unavailable.".to_string())
}

fn emit_ssh_event(app_handle: &tauri::AppHandle, session_id: &str, kind: &str, text: &str) {
    if text.is_empty() {
        return;
    }

    let payload = SshStreamEvent {
        session_id: session_id.to_string(),
        kind: kind.to_string(),
        text: text.to_string(),
        at: Utc::now().to_rfc3339(),
    };
    if let Err(err) = app_handle.emit("ssh-output", payload) {
        warn!("Failed to emit ssh-output event: {}", err);
    }
}

/// Connect to an ECS instance over SSH and store the live session.
#[tauri::command]
async fn ssh_connect(
    params: SshConnectParams,
    state: tauri::State<'_, SshSessionStore>,
    app_handle: tauri::AppHandle,
) -> Result<SshConnectResult, String> {
    let session_id = normalize_ssh_session_id(&params.session_id)?;
    let host = params.host.trim().to_string();
    if host.is_empty() {
        return Err("SSH host is required.".to_string());
    }

    let port = params.port.unwrap_or(22);
    let username = params
        .username
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("root")
        .to_string();
    let password = params.password.trim().to_string();
    if password.is_empty() {
        return Err("SSH password is required.".to_string());
    }

    let existing = {
        let mut sessions = lock_ssh_sessions(&state)?;
        sessions.remove(&session_id)
    };
    if let Some(stale) = existing {
        stale.shell_reader_task.abort();
        let _ = stale
            .handle
            .disconnect(Disconnect::ByApplication, "", "en")
            .await;
    }

    let config = Arc::new(client::Config {
        inactivity_timeout: Some(Duration::from_secs(30)),
        ..<_>::default()
    });
    let mut handle = client::connect(config, (host.as_str(), port), SshClientHandler)
        .await
        .map_err(|err| format!("SSH connection failed to {}:{}: {}", host, port, err))?;
    let auth = handle
        .authenticate_password(username.clone(), password)
        .await
        .map_err(|err| {
            format!(
                "SSH authentication failed for {}@{}:{}: {}",
                username, host, port, err
            )
        })?;
    if !auth.success() {
        return Err(format!(
            "SSH authentication rejected for {}@{}:{}.",
            username, host, port
        ));
    }

    let channel = handle
        .channel_open_session()
        .await
        .map_err(|err| format!("Failed to open interactive SSH channel: {}", err))?;
    channel
        .request_pty(false, "xterm-256color", 220, 64, 0, 0, &[])
        .await
        .map_err(|err| format!("Failed to request SSH PTY: {}", err))?;
    channel
        .request_shell(false)
        .await
        .map_err(|err| format!("Failed to request SSH shell: {}", err))?;
    let (mut shell_reader, shell_writer) = channel.split();

    let session_id_for_task = session_id.clone();
    let app_handle_for_task = app_handle.clone();
    let shell_reader_task = tokio::spawn(async move {
        while let Some(message) = shell_reader.wait().await {
            match message {
                ChannelMsg::Data { data } => {
                    emit_ssh_event(
                        &app_handle_for_task,
                        &session_id_for_task,
                        "stdout",
                        &String::from_utf8_lossy(data.as_ref()),
                    );
                }
                ChannelMsg::ExtendedData { data, .. } => {
                    emit_ssh_event(
                        &app_handle_for_task,
                        &session_id_for_task,
                        "stderr",
                        &String::from_utf8_lossy(data.as_ref()),
                    );
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    emit_ssh_event(
                        &app_handle_for_task,
                        &session_id_for_task,
                        "meta",
                        &format!("Exit status: {}", exit_status),
                    );
                }
                ChannelMsg::Eof => {
                    emit_ssh_event(
                        &app_handle_for_task,
                        &session_id_for_task,
                        "meta",
                        "Remote shell sent EOF.",
                    );
                }
                ChannelMsg::Close => {
                    emit_ssh_event(
                        &app_handle_for_task,
                        &session_id_for_task,
                        "meta",
                        "Remote shell closed.",
                    );
                    break;
                }
                _ => {}
            }
        }
    });

    let connected_at = Utc::now().to_rfc3339();
    {
        let mut sessions = lock_ssh_sessions(&state)?;
        sessions.insert(
            session_id.clone(),
            SshSessionEntry {
                handle,
                shell_writer,
                shell_reader_task,
                host: host.clone(),
                port,
                username: username.clone(),
            },
        );
    }

    info!(
        "SSH connected: session_id={} target={}@{}:{}",
        session_id, username, host, port
    );

    Ok(SshConnectResult {
        session_id,
        host,
        port,
        username,
        connected_at,
    })
}

/// Execute one command over an existing SSH connection.
#[tauri::command]
async fn ssh_exec(
    params: SshExecParams,
    state: tauri::State<'_, SshSessionStore>,
) -> Result<SshExecResult, String> {
    let session_id = normalize_ssh_session_id(&params.session_id)?;
    let command = params.command.trim().to_string();
    if command.is_empty() {
        return Err("SSH command is required.".to_string());
    }

    let entry = {
        let mut sessions = lock_ssh_sessions(&state)?;
        sessions
            .remove(&session_id)
            .ok_or_else(|| format!("No SSH connection found for session {}.", session_id))?
    };

    info!(
        "Running SSH command: session_id={} target={}@{}:{} command={}",
        session_id, entry.username, entry.host, entry.port, command
    );

    let payload = format!("{}\n", command);
    let send_result = entry
        .shell_writer
        .data(Cursor::new(payload.into_bytes()))
        .await
        .map_err(|err| format!("Failed to send command to live SSH shell: {}", err));

    if let Err(err) = send_result {
        entry.shell_reader_task.abort();
        warn!(
            "SSH command failed; dropping session_id={} target={}@{}:{} error={}",
            session_id, entry.username, entry.host, entry.port, err
        );
        return Err(err);
    }

    let mut sessions = lock_ssh_sessions(&state)?;
    sessions.insert(session_id.clone(), entry);

    Ok(SshExecResult {
        session_id,
        command,
        stdout: String::new(),
        stderr: String::new(),
        exit_status: None,
    })
}

/// Resize the PTY for an existing SSH shell session.
#[tauri::command]
async fn ssh_resize(
    params: SshResizeParams,
    state: tauri::State<'_, SshSessionStore>,
) -> Result<SshResizeResult, String> {
    let session_id = normalize_ssh_session_id(&params.session_id)?;
    let cols = params.cols.clamp(40, 400);
    let rows = params.rows.clamp(10, 180);
    let pixel_width = params.pixel_width.unwrap_or(0);
    let pixel_height = params.pixel_height.unwrap_or(0);

    let entry = {
        let mut sessions = lock_ssh_sessions(&state)?;
        sessions
            .remove(&session_id)
            .ok_or_else(|| format!("No SSH connection found for session {}.", session_id))?
    };

    let resize_result = entry
        .shell_writer
        .window_change(cols, rows, pixel_width, pixel_height)
        .await
        .map_err(|err| {
            format!(
                "Failed to resize SSH PTY for session {}: {}",
                session_id, err
            )
        });

    if let Err(err) = resize_result {
        entry.shell_reader_task.abort();
        warn!(
            "SSH PTY resize failed; dropping session_id={} target={}@{}:{} error={}",
            session_id, entry.username, entry.host, entry.port, err
        );
        return Err(err);
    }

    let mut sessions = lock_ssh_sessions(&state)?;
    sessions.insert(session_id.clone(), entry);

    Ok(SshResizeResult {
        session_id,
        cols,
        rows,
    })
}

/// Send interactive control bytes (Ctrl+C/Ctrl+D/Ctrl+U) to an SSH shell session.
#[tauri::command]
async fn ssh_send_control(
    params: SshSendControlParams,
    state: tauri::State<'_, SshSessionStore>,
) -> Result<SshSendControlResult, String> {
    let session_id = normalize_ssh_session_id(&params.session_id)?;
    let control = params.control.trim().to_string();
    let control_byte = control_char_from_input(&control)?;

    let entry = {
        let mut sessions = lock_ssh_sessions(&state)?;
        sessions
            .remove(&session_id)
            .ok_or_else(|| format!("No SSH connection found for session {}.", session_id))?
    };

    let payload = vec![control_byte];
    let send_result = entry
        .shell_writer
        .data(Cursor::new(payload))
        .await
        .map_err(|err| {
            format!(
                "Failed to send {} to SSH session {}: {}",
                control, session_id, err
            )
        });

    if let Err(err) = send_result {
        entry.shell_reader_task.abort();
        warn!(
            "SSH control send failed; dropping session_id={} target={}@{}:{} error={}",
            session_id, entry.username, entry.host, entry.port, err
        );
        return Err(err);
    }

    let mut sessions = lock_ssh_sessions(&state)?;
    sessions.insert(session_id.clone(), entry);

    Ok(SshSendControlResult {
        session_id,
        control,
        sent: true,
    })
}

/// Execute one remote command by creating a short-lived SSH connection.
#[tauri::command]
async fn ssh_exec_one_shot(
    params: SshExecOneShotParams,
    app_handle: tauri::AppHandle,
) -> Result<SshExecOneShotResult, String> {
    let session_id = normalize_ssh_session_id(&params.session_id)?;
    let host = params.host.trim().to_string();
    if host.is_empty() {
        return Err("SSH host is required.".to_string());
    }

    let command = params.command.trim().to_string();
    if command.is_empty() {
        return Err("SSH command is required.".to_string());
    }

    let port = params.port.unwrap_or(22);
    let username = params
        .username
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("root")
        .to_string();
    let password = params.password.trim().to_string();
    if password.is_empty() {
        return Err("SSH password is required.".to_string());
    }

    let config = Arc::new(client::Config {
        inactivity_timeout: Some(Duration::from_secs(60)),
        ..<_>::default()
    });
    let mut handle = client::connect(config, (host.as_str(), port), SshClientHandler)
        .await
        .map_err(|err| format!("SSH connection failed to {}:{}: {}", host, port, err))?;
    let auth = handle
        .authenticate_password(username.clone(), password)
        .await
        .map_err(|err| {
            format!(
                "SSH authentication failed for {}@{}:{}: {}",
                username, host, port, err
            )
        })?;
    if !auth.success() {
        return Err(format!(
            "SSH authentication rejected for {}@{}:{}.",
            username, host, port
        ));
    }

    let mut channel = handle
        .channel_open_session()
        .await
        .map_err(|err| format!("Failed to open SSH exec channel: {}", err))?;
    channel
        .request_pty(false, "xterm-256color", 220, 64, 0, 0, &[])
        .await
        .map_err(|err| format!("Failed to request SSH PTY: {}", err))?;
    channel
        .exec(true, command.clone())
        .await
        .map_err(|err| format!("Failed to execute remote command: {}", err))?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut exit_status: Option<u32> = None;
    while let Some(message) = channel.wait().await {
        match message {
            ChannelMsg::Data { data } => {
                let text = String::from_utf8_lossy(data.as_ref()).to_string();
                stdout.push_str(&text);
                emit_ssh_event(&app_handle, &session_id, "stdout", &text);
            }
            ChannelMsg::ExtendedData { data, .. } => {
                let text = String::from_utf8_lossy(data.as_ref()).to_string();
                stderr.push_str(&text);
                emit_ssh_event(&app_handle, &session_id, "stderr", &text);
            }
            ChannelMsg::ExitStatus {
                exit_status: remote_status,
            } => {
                exit_status = Some(remote_status);
                emit_ssh_event(
                    &app_handle,
                    &session_id,
                    "meta",
                    &format!("Exit status: {}", remote_status),
                );
            }
            ChannelMsg::Eof => {
                emit_ssh_event(&app_handle, &session_id, "meta", "Remote command sent EOF.");
            }
            ChannelMsg::Close => {
                emit_ssh_event(
                    &app_handle,
                    &session_id,
                    "meta",
                    "Remote command channel closed.",
                );
            }
            _ => {}
        }
    }
    let _ = channel.eof().await;
    let _ = channel.close().await;
    if let Err(err) = handle.disconnect(Disconnect::ByApplication, "", "en").await {
        warn!(
            "SSH one-shot disconnect returned error: target={}@{}:{} error={}",
            username, host, port, err
        );
    }

    Ok(SshExecOneShotResult {
        session_id,
        host,
        port,
        username,
        command,
        stdout,
        stderr,
        exit_status,
    })
}

/// Disconnect and remove one SSH session.
#[tauri::command]
async fn ssh_disconnect(
    params: SshDisconnectParams,
    state: tauri::State<'_, SshSessionStore>,
) -> Result<SshDisconnectResult, String> {
    let session_id = normalize_ssh_session_id(&params.session_id)?;
    let existing = {
        let mut sessions = lock_ssh_sessions(&state)?;
        sessions.remove(&session_id)
    };

    if let Some(session) = existing {
        let _ = session.shell_writer.close().await;
        session.shell_reader_task.abort();
        if let Err(err) = session
            .handle
            .disconnect(Disconnect::ByApplication, "", "en")
            .await
        {
            warn!(
                "SSH disconnect returned error: session_id={} target={}@{}:{} error={}",
                session_id, session.username, session.host, session.port, err
            );
        }

        info!(
            "SSH disconnected: session_id={} target={}@{}:{}",
            session_id, session.username, session.host, session.port
        );
        return Ok(SshDisconnectResult {
            session_id,
            disconnected: true,
        });
    }

    Ok(SshDisconnectResult {
        session_id,
        disconnected: false,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(SshSessionStore::default())
        .invoke_handler(tauri::generate_handler![
            list_vpcs,
            list_subnets,
            list_images,
            list_flavors,
            list_eips,
            list_ecses,
            list_evss,
            list_cce_clusters,
            create_cce_cluster,
            delete_cce_cluster,
            list_cce_node_pools,
            get_cce_job,
            list_cce_nat_gateways,
            create_cce_nat_gateway,
            delete_cce_nat_gateway,
            bind_cce_cluster_api_eip,
            get_cce_cluster_kubeconfig,
            list_obs_buckets,
            create_obs_bucket,
            delete_obs_bucket,
            list_obs_objects,
            put_obs_object,
            get_obs_object,
            delete_obs_object,
            create_ecs,
            delete_ecs_with_eip,
            stop_ecs,
            ssh_connect,
            ssh_exec,
            ssh_resize,
            ssh_send_control,
            ssh_exec_one_shot,
            ssh_disconnect
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{
        extract_cluster_kubeconfig, extract_eip_id_and_address, extract_nat_gateway_id,
        normalize_server_name, RANDOM_NAME_PLACEHOLDER,
    };

    #[test]
    fn normalize_server_name_keeps_custom_value() {
        assert_eq!(normalize_server_name("my-ecs-prod"), "my-ecs-prod");
    }

    #[test]
    fn normalize_server_name_generates_when_placeholder_or_blank() {
        let from_placeholder = normalize_server_name(RANDOM_NAME_PLACEHOLDER);
        let from_blank = normalize_server_name("");

        assert!(from_placeholder.starts_with("ecs-"));
        assert!(from_blank.starts_with("ecs-"));
        assert_ne!(from_placeholder, from_blank);
    }

    #[test]
    fn extract_nat_gateway_id_reads_nested_payload() {
        let raw = r#"{"nat_gateway":{"id":"nat-123","name":"cce-nat"}}"#;
        assert_eq!(extract_nat_gateway_id(raw).as_deref(), Some("nat-123"));
    }

    #[test]
    fn extract_eip_id_and_address_reads_publicip_payload() {
        let raw = r#"{"publicip":{"id":"eip-123","public_ip_address":"1.2.3.4"}}"#;
        let (id, address) = extract_eip_id_and_address(raw);
        assert_eq!(id.as_deref(), Some("eip-123"));
        assert_eq!(address.as_deref(), Some("1.2.3.4"));
    }

    #[test]
    fn extract_cluster_kubeconfig_reads_common_fields() {
        let raw = r#"{"kubeconfig":"apiVersion: v1\nclusters: []"}"#;
        assert_eq!(
            extract_cluster_kubeconfig(raw).as_deref(),
            Some("apiVersion: v1\nclusters: []")
        );
    }

    #[test]
    fn extract_cluster_kubeconfig_reads_nested_certs_fields() {
        let raw = r#"{"certs":{"kube_config":"apiVersion: v1\nclusters: []"}}"#;
        assert_eq!(
            extract_cluster_kubeconfig(raw).as_deref(),
            Some("apiVersion: v1\nclusters: []")
        );
    }
}
