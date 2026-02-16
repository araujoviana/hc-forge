mod api;

use crate::api::models::eip::EipListResponse;
use crate::api::models::evs::EvsListResponse;
use crate::api::models::ims::Image;
use api::models::ecs::{
    Bandwidth, CreateEcsRequest, DataVolume, EcsListResponse, Eip, Flavor, Nic, PublicIp,
    RootVolume, Server,
};
use api::models::vpc::{Subnet, Vpc};
use api::{Credentials, CredentialsSource, HwcClient, ImageListFilters, ListParams};
use chrono::Utc;
use log::{error, info, warn};
use rand::{distr::Alphanumeric, Rng};
use russh::{client, ChannelMsg, Disconnect};
use serde::{Deserialize, Serialize};
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

fn normalize_ssh_session_id(input: &str) -> Result<String, String> {
    let session_id = input.trim();
    if session_id.is_empty() {
        return Err("SSH session ID is required.".to_string());
    }
    Ok(session_id.to_string())
}

fn control_char_from_input(input: &str) -> Result<u8, String> {
    let normalized = input.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "c" | "ctrl+c" => Ok(0x03),
        "d" | "ctrl+d" => Ok(0x04),
        "u" | "ctrl+u" => Ok(0x15),
        _ => Err(format!(
            "Unsupported control sequence '{}'. Use Ctrl+C, Ctrl+D, or Ctrl+U.",
            input.trim()
        )),
    }
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
                break;
            }
            ChannelMsg::Close => {
                emit_ssh_event(
                    &app_handle,
                    &session_id,
                    "meta",
                    "Remote command channel closed.",
                );
                break;
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
    use super::{normalize_server_name, RANDOM_NAME_PLACEHOLDER};

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
}
