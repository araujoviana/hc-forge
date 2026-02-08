use anyhow::{Context, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{Client, Method, Request, StatusCode};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use log::{debug, warn};

use super::auth::credentials::Credentials;
use super::models::ecs::CreateEcsRequest;
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
    pub async fn list_images(&self, region: &str) -> Result<Vec<Image>> {
        let host = format!("ims.{region}.myhuaweicloud.com");
        let path = "/v2/cloudimages".to_string();

        let body: ImageListResponse = self
            .send_json(Method::GET, &host, &path, None)
            .await
            .context("Failed to list images")?;

        Ok(body.images)
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
            req = req.header(HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON).body(json);
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
