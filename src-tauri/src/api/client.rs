use anyhow::{Context, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use log::{debug, warn};
use reqwest::{Client, Method, Request, StatusCode};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use super::auth::credentials::Credentials;
use super::models::ecs::{
    CreateEcsRequest, DeleteEcsRequest, DeleteEcsServer, EcsListResponse, Flavor,
    FlavorListResponse, StopEcsAction, StopEcsRequest, StopEcsServer,
};
use super::models::eip::EipListResponse;
use super::models::evs::EvsListResponse;
use super::models::iam::ProjectsResponse;
use super::models::ims::{Image, ImageListResponse};
use super::models::vpc::{Subnet, SubnetListResponse, Vpc, VpcListResponse};

type HmacSha256 = Hmac<Sha256>;

const SIGNING_ALGORITHM: &str = "SDK-HMAC-SHA256";
const SIGNED_HEADERS: &str = "host;x-sdk-date";
const HEADER_HOST: &str = "Host";
const HEADER_DATE: &str = "X-Sdk-Date";
const HEADER_AUTH: &str = "Authorization";
const HEADER_CONTENT_TYPE: &str = "Content-Type";
const CONTENT_TYPE_JSON: &str = "application/json";
const IAM_PROJECTS_PATH: &str = "/v3/auth/projects";

/// Minimal Huawei Cloud API client with request signing.
pub struct HwcClient {
    credentials: Credentials,
    http: Client,
}

#[derive(Debug, Clone, Default)]
pub struct ImageListFilters {
    pub visibility: Option<String>,
    pub image_type: Option<String>,
    pub flavor_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub marker: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

fn push_query_param(params: &mut Vec<String>, key: &str, value: &str) {
    if !value.is_empty() {
        params.push(format!("{key}={value}"));
    }
}

fn push_query_param_u32(params: &mut Vec<String>, key: &str, value: Option<u32>) {
    if let Some(value) = value {
        params.push(format!("{key}={value}"));
    }
}

impl HwcClient {
    pub fn new(credentials: Credentials) -> Self {
        Self {
            credentials,
            http: Client::new(),
        }
    }

    /// List VPCs for the given region.
    pub async fn list_vpcs(&self, region: &str) -> Result<Vec<Vpc>> {
        let project_id = self.project_id(region).await?;
        let host = format!("vpc.{region}.myhuaweicloud.com");
        let path = format!("/v1/{project_id}/vpcs");

        let body: VpcListResponse = self
            .send_json(Method::GET, &host, &path, None)
            .await
            .context("Failed to list VPCs")?;

        Ok(body.vpcs)
    }

    /// List subnets for the selected VPC.
    pub async fn list_subnets(&self, region: &str, vpc_id: &str) -> Result<Vec<Subnet>> {
        let project_id = self.project_id(region).await?;
        let host = format!("vpc.{region}.myhuaweicloud.com");
        let path = format!("/v1/{project_id}/subnets?vpc_id={vpc_id}");

        let body: SubnetListResponse = self
            .send_json(Method::GET, &host, &path, None)
            .await
            .context("Failed to list subnets")?;

        Ok(body.subnets)
    }

    /// List images for the given region.
    /// IMS Querying Images: GET https://{Endpoint}/v2/cloudimages
    pub async fn list_images(
        &self,
        region: &str,
        filters: Option<ImageListFilters>,
    ) -> Result<Vec<Image>> {
        let host = format!("ims.{region}.myhuaweicloud.com");
        let mut params = vec!["virtual_env_type=FusionCompute".to_string()];

        if let Some(filters) = filters {
            if let Some(visibility) = filters.visibility.as_deref() {
                push_query_param(&mut params, "visibility", visibility);
            }
            if let Some(image_type) = filters.image_type.as_deref() {
                push_query_param(&mut params, "__imagetype", image_type);
            }
            if let Some(flavor_id) = filters.flavor_id.as_deref() {
                push_query_param(&mut params, "flavor_id", flavor_id);
            }
        }

        let path = if params.is_empty() {
            "/v2/cloudimages".to_string()
        } else {
            format!("/v2/cloudimages?{}", params.join("&"))
        };

        let body: ImageListResponse = self
            .send_json(Method::GET, &host, &path, None)
            .await
            .context("Failed to list images")?;

        Ok(body.images)
    }

    /// List flavors for the given region.
    /// ECS Querying Flavors: GET https://{Endpoint}/v1/{project_id}/cloudservers/flavors
    pub async fn list_flavors(&self, region: &str) -> Result<Vec<Flavor>> {
        let project_id = self.project_id(region).await?;
        let host = format!("ecs.{region}.myhuaweicloud.com");
        let path = format!("/v1/{project_id}/cloudservers/flavors?limit=1000");

        let body: FlavorListResponse = self
            .send_json(Method::GET, &host, &path, None)
            .await
            .context("Failed to list flavors")?;

        Ok(body.flavors)
    }

    /// List elastic IPs for the given region.
    /// EIP Querying Elastic IPs: GET /v3/{project_id}/eip/publicips
    pub async fn list_eips(
        &self,
        region: &str,
        params: Option<ListParams>,
    ) -> Result<EipListResponse> {
        let project_id = self.project_id(region).await?;
        // EIP uses the same endpoint as VPC per Huawei Cloud docs.
        let host = format!("vpc.{region}.myhuaweicloud.com");
        let mut query: Vec<String> = Vec::new();

        if let Some(params) = params {
            if let Some(marker) = params.marker.as_deref() {
                push_query_param(&mut query, "marker", marker);
            }
            push_query_param_u32(&mut query, "limit", params.limit);
            push_query_param_u32(&mut query, "offset", params.offset);
        }

        let query_string = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };

        let path_v3 = format!("/v3/{project_id}/eip/publicips{query_string}");
        match self.send_json(Method::GET, &host, &path_v3, None).await {
            Ok(response) => Ok(response),
            Err(err) => {
                warn!("List EIPs v3 failed, falling back to v1: {}", err);
                let path_v1 = format!("/v1/{project_id}/publicips{query_string}");
                self.send_json(Method::GET, &host, &path_v1, None)
                    .await
                    .context("Failed to list EIPs (v1 fallback)")
            }
        }
    }

    /// List ECS servers for the given region.
    /// ECS Querying ECS Detail: GET /v1.1/{project_id}/cloudservers/detail
    pub async fn list_ecses(
        &self,
        region: &str,
        params: Option<ListParams>,
    ) -> Result<EcsListResponse> {
        let project_id = self.project_id(region).await?;
        let host = format!("ecs.{region}.myhuaweicloud.com");
        let mut query: Vec<String> = Vec::new();

        if let Some(params) = params {
            if let Some(marker) = params.marker.as_deref() {
                push_query_param(&mut query, "marker", marker);
            }
            push_query_param_u32(&mut query, "limit", params.limit);
        }

        let base_path = format!("/v1.1/{project_id}/cloudservers/detail");
        let path = if query.is_empty() {
            base_path
        } else {
            format!("{base_path}?{}", query.join("&"))
        };

        self.send_json(Method::GET, &host, &path, None)
            .await
            .context("Failed to list ECSes")
    }

    /// List EVS disks for the given region.
    /// EVS Querying Details About EVS Disks: GET /v2/{project_id}/cloudvolumes/detail
    pub async fn list_evss(
        &self,
        region: &str,
        params: Option<ListParams>,
    ) -> Result<EvsListResponse> {
        let project_id = self.project_id(region).await?;
        let host = format!("evs.{region}.myhuaweicloud.com");
        let mut query: Vec<String> = Vec::new();

        if let Some(params) = params {
            if let Some(marker) = params.marker.as_deref() {
                push_query_param(&mut query, "marker", marker);
            }
            push_query_param_u32(&mut query, "limit", params.limit);
            push_query_param_u32(&mut query, "offset", params.offset);
        }

        let base_path = format!("/v2/{project_id}/cloudvolumes/detail");
        let path = if query.is_empty() {
            base_path
        } else {
            format!("{base_path}?{}", query.join("&"))
        };

        self.send_json(Method::GET, &host, &path, None)
            .await
            .context("Failed to list EVS disks")
    }

    /// Create an ECS instance and return the status + raw response body.
    pub async fn create_ecs(
        &self,
        region: &str,
        body: &CreateEcsRequest,
    ) -> Result<(StatusCode, String)> {
        let project_id = self.project_id(region).await?;
        let host = format!("ecs.{region}.myhuaweicloud.com");
        let path = format!("/v1/{project_id}/cloudservers");
        let json = serde_json::to_string(body).context("Failed to serialize ECS payload")?;

        self.send_raw(Method::POST, &host, &path, Some(json)).await
    }

    /// Delete an ECS instance.
    /// ECS Deleting ECSs in Batches: POST /v1/{project_id}/cloudservers/delete
    pub async fn delete_ecs(
        &self,
        region: &str,
        server_id: &str,
        delete_publicip: bool,
        delete_volume: bool,
    ) -> Result<(StatusCode, String)> {
        let project_id = self.project_id(region).await?;
        let host = format!("ecs.{region}.myhuaweicloud.com");
        let path = format!("/v1/{project_id}/cloudservers/delete");
        let payload = DeleteEcsRequest {
            servers: vec![DeleteEcsServer {
                id: server_id.to_string(),
            }],
            delete_publicip: Some(delete_publicip),
            delete_volume: Some(delete_volume),
        };
        let json =
            serde_json::to_string(&payload).context("Failed to serialize ECS delete payload")?;

        self.send_raw(Method::POST, &host, &path, Some(json)).await
    }

    /// Delete an Elastic IP.
    /// EIP Deleting an Elastic IP: DELETE /v1/{project_id}/publicips/{publicip_id}
    pub async fn delete_eip(&self, region: &str, eip_id: &str) -> Result<(StatusCode, String)> {
        let project_id = self.project_id(region).await?;
        let host = format!("vpc.{region}.myhuaweicloud.com");

        // Try v3 first; fall back to v1 where needed.
        let path_v3 = format!("/v3/{project_id}/eip/publicips/{eip_id}");
        let (status_v3, body_v3) = self.send_raw(Method::DELETE, &host, &path_v3, None).await?;
        if status_v3 != StatusCode::NOT_FOUND && status_v3 != StatusCode::METHOD_NOT_ALLOWED {
            return Ok((status_v3, body_v3));
        }

        let path_v1 = format!("/v1/{project_id}/publicips/{eip_id}");
        self.send_raw(Method::DELETE, &host, &path_v1, None).await
    }

    /// Stop ECS instances.
    /// ECS Batch Stop: POST /v1/{project_id}/cloudservers/action with os-stop body.
    pub async fn stop_ecs(
        &self,
        region: &str,
        server_id: &str,
        stop_type: &str,
    ) -> Result<(StatusCode, String)> {
        let project_id = self.project_id(region).await?;
        let host = format!("ecs.{region}.myhuaweicloud.com");
        let path = format!("/v1/{project_id}/cloudservers/action");
        let payload = StopEcsRequest {
            os_stop: StopEcsAction {
                servers: vec![StopEcsServer {
                    id: server_id.to_string(),
                }],
                stop_type: stop_type.to_string(),
            },
        };
        let json =
            serde_json::to_string(&payload).context("Failed to serialize ECS stop payload")?;

        self.send_raw(Method::POST, &host, &path, Some(json)).await
    }

    /// Resolve project ID for the provided region.
    async fn project_id(&self, region: &str) -> Result<String> {
        let host = format!("iam.{region}.myhuaweicloud.com");
        let body: ProjectsResponse = self
            .send_json(Method::GET, &host, IAM_PROJECTS_PATH, None)
            .await
            .context("Failed to query projects")?;

        let project = body
            .projects
            .iter()
            .find(|p| p.enabled && p.name == region)
            .ok_or_else(|| {
                let available = body
                    .projects
                    .iter()
                    .filter(|p| p.enabled)
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");

                anyhow::anyhow!(
                    "No enabled project found for region '{}'. Enabled projects: {}",
                    region,
                    if available.is_empty() {
                        "<none>"
                    } else {
                        &available
                    }
                )
            })?;

        Ok(project.id.clone())
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        method: Method,
        host: &str,
        path: &str,
        body: Option<String>,
    ) -> Result<T> {
        let req = self.build_request(method, host, path, body)?;
        let resp = self.http.execute(req).await.context("Request failed")?;
        let status = resp.status();
        let text = resp.text().await.context("Failed to read response")?;

        if !status.is_success() {
            warn!(
                "Huawei Cloud API error: status={} host={} path={} body={}",
                status, host, path, text
            );
            anyhow::bail!("Huawei Cloud API returned {}", status);
        }

        serde_json::from_str(&text).context("Failed to parse JSON response")
    }

    async fn send_raw(
        &self,
        method: Method,
        host: &str,
        path: &str,
        body: Option<String>,
    ) -> Result<(StatusCode, String)> {
        let req = self.build_request(method, host, path, body)?;
        let resp = self.http.execute(req).await.context("Request failed")?;
        let status = resp.status();
        let text = resp.text().await.context("Failed to read response")?;

        if !status.is_success() {
            warn!(
                "Huawei Cloud API error: status={} host={} path={} body={}",
                status, host, path, text
            );
        }

        Ok((status, text))
    }

    /// Build a signed HTTP request using the Huawei Cloud SDK-HMAC-SHA256 scheme.
    fn build_request(
        &self,
        method: Method,
        host: &str,
        path: &str,
        body: Option<String>,
    ) -> Result<Request> {
        let url = format!("https://{host}{path}");
        let x_sdk_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

        let (raw_path, raw_query) = split_path_query(path);
        let canonical_path = canonicalize_path(raw_path);
        let canonical_query = canonicalize_query(raw_query);
        let payload_hash = sha256_hex(body.as_deref().unwrap_or(""));

        let canonical_request = format!(
            "{}\n{}\n{}\nhost:{}\nx-sdk-date:{}\n\n{}\n{}",
            method.as_str().to_uppercase(),
            canonical_path,
            canonical_query,
            host,
            x_sdk_date,
            SIGNED_HEADERS,
            payload_hash
        );

        let string_to_sign = format!(
            "{}\n{}\n{}",
            SIGNING_ALGORITHM,
            x_sdk_date,
            sha256_hex(&canonical_request)
        );

        let mut mac = HmacSha256::new_from_slice(self.credentials.secret_key.as_bytes())?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        let authorization = format!(
            "{} Access={}, SignedHeaders={}, Signature={}",
            SIGNING_ALGORITHM, self.credentials.access_key, SIGNED_HEADERS, signature
        );

        let mut req = self
            .http
            .request(method, url)
            .header(HEADER_HOST, host)
            .header(HEADER_DATE, x_sdk_date)
            .header(HEADER_AUTH, authorization);

        if let Some(json) = body {
            req = req
                .header(HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON)
                .body(json);
        }

        debug!("Signed Huawei Cloud request: host={} path={}", host, path);
        Ok(req.build()?)
    }
}

fn split_path_query(path: &str) -> (&str, Option<&str>) {
    match path.split_once('?') {
        Some((raw_path, raw_query)) => (raw_path, Some(raw_query)),
        None => (path, None),
    }
}

fn canonicalize_path(path: &str) -> String {
    let mut encoded = String::new();

    if path.is_empty() {
        encoded.push('/');
        return encoded;
    }

    for b in path.as_bytes() {
        if *b == b'/' {
            encoded.push('/');
        } else if is_unreserved(*b) {
            encoded.push(*b as char);
        } else {
            encoded.push_str(&percent_encode_byte(*b));
        }
    }

    if !encoded.ends_with('/') {
        encoded.push('/');
    }

    encoded
}

fn canonicalize_query(query: Option<&str>) -> String {
    let Some(query) = query else {
        return String::new();
    };

    let mut pairs: Vec<(String, String)> = Vec::new();

    for part in query.split('&') {
        if part.is_empty() {
            continue;
        }

        let (name, value) = match part.split_once('=') {
            Some((n, v)) => (n, v),
            None => (part, ""),
        };

        pairs.push((encode_rfc3986(name), encode_rfc3986(value)));
    }

    pairs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    pairs
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}

fn encode_rfc3986(input: &str) -> String {
    let mut encoded = String::new();

    for b in input.as_bytes() {
        if is_unreserved(*b) {
            encoded.push(*b as char);
        } else {
            encoded.push_str(&percent_encode_byte(*b));
        }
    }

    encoded
}

fn is_unreserved(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~')
}

fn percent_encode_byte(byte: u8) -> String {
    format!("%{:02X}", byte)
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{canonicalize_path, canonicalize_query};

    #[test]
    fn canonicalize_path_encodes_reserved_and_appends_trailing_slash() {
        assert_eq!(
            canonicalize_path("/v1/project id/cloudservers"),
            "/v1/project%20id/cloudservers/"
        );
    }

    #[test]
    fn canonicalize_query_sorts_and_encodes_params() {
        let actual = canonicalize_query(Some("z=last&name=a b&a=first"));
        assert_eq!(actual, "a=first&name=a%20b&z=last");
    }

    #[test]
    fn canonicalize_query_handles_missing_values() {
        let actual = canonicalize_query(Some("foo&bar=baz"));
        assert_eq!(actual, "bar=baz&foo=");
    }
}
