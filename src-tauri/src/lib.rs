mod api;

use api::{load_credentials, Credentials, CredentialsSource, HwcClient};
use api::models::ecs::{Bandwidth, CreateEcsRequest, Eip, Nic, PublicIp, RootVolume, Server};
use api::models::vpc::{Subnet, Vpc};
use chrono::Utc;
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use log::{error, info};

const RANDOM_NAME_PLACEHOLDER: &str = "ecs-<RANDOM-VALUE>";
const DEFAULT_EIP_TYPE: &str = "5_bgp";
const DEFAULT_BANDWIDTH_SHARE_TYPE: &str = "PER";
const DEFAULT_BANDWIDTH_CHARGE_MODE: &str = "traffic";
const DEFAULT_BANDWIDTH_SIZE: u32 = 1;

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
}

/// AK/SK credentials input from the UI, or empty to use env/file defaults.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CredentialsInput {
    access_key: String,
    secret_key: String,
}

#[derive(Debug, Serialize)]
struct CreateEcsResult {
    status: String,
    status_code: u16,
    body: String,
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

// Choose credentials from the UI if provided, otherwise fall back to env/file defaults.
fn resolve_credentials(
    input: Option<CredentialsInput>,
) -> Result<(Credentials, CredentialsSource), String> {
    if let Some(input) = input {
        let access_key = input.access_key.trim();
        let secret_key = input.secret_key.trim();

        if access_key.is_empty() && secret_key.is_empty() {
            return load_credentials().map_err(|err| err.to_string());
        }

        if access_key.is_empty() || secret_key.is_empty() {
            return Err(
                "Provide both Access Key and Secret Key, or leave both blank to use defaults."
                    .to_string(),
            );
        }

        return Ok((
            Credentials::new(access_key.to_string(), secret_key.to_string()),
            CredentialsSource::Explicit,
        ));
    }

    load_credentials().map_err(|err| err.to_string())
}

fn credentials_source_label(source: &CredentialsSource) -> String {
    match source {
        CredentialsSource::Environment => "env".to_string(),
        CredentialsSource::File(path) => format!("file:{}", path.display()),
        CredentialsSource::Explicit => "explicit".to_string(),
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
    client
        .list_vpcs(&region)
        .await
        .map_err(|err| {
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
    client
        .list_subnets(&region, &vpc_id)
        .await
        .map_err(|err| {
            error!(
                "Failed to list subnets: region={} vpc_id={} error={}",
                region, vpc_id, err
            );
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

    let publicip = if params.eip {
        Some(PublicIp {
            eip: Eip {
                ip_type: DEFAULT_EIP_TYPE.into(),
                bandwidth: Bandwidth {
                    size: DEFAULT_BANDWIDTH_SIZE,
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
            publicip,
        },
    };

    let client = HwcClient::new(credentials);
    let (status, body) = client
        .create_ecs(&params.region, &body)
        .await
        .map_err(|err| {
            error!("Failed to create ECS: region={} error={}", params.region, err);
            err.to_string()
        })?;

    Ok(CreateEcsResult {
        status: status.to_string(),
        status_code: status.as_u16(),
        body,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_vpcs,
            list_subnets,
            create_ecs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
