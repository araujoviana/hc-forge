/// Module for loading Huawei Cloud credentials.
use anyhow::{Context, Result};
use directories::{ProjectDirs, UserDirs};
use std::{env, fs, path::{Path, PathBuf}};

const ENV_ACCESS_KEY: &str = "HWC_AK";
const ENV_SECRET_KEY: &str = "HWC_SK";
const ENV_CREDENTIALS_FILE: &str = "HWC_CREDENTIALS_FILE";
const DEFAULT_CREDENTIALS_FILE: &str = "credentials.csv";

#[derive(Clone, Debug)]
pub struct Credentials {
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
}

impl Credentials {
    pub fn new(access_key: String, secret_key: String) -> Self {
        Self {
            access_key,
            secret_key,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CredentialsSource {
    Environment,
    File(PathBuf),
    Explicit,
}

/// Load credentials from environment variables or a credentials.csv file.
pub fn load_credentials() -> Result<(Credentials, CredentialsSource)> {
    if let (Ok(ak), Ok(sk)) = (env::var(ENV_ACCESS_KEY), env::var(ENV_SECRET_KEY)) {
        return Ok((Credentials::new(ak, sk), CredentialsSource::Environment));
    }

    let candidates = credentials_file_candidates();
    for path in candidates {
        if path.exists() {
            let creds = load_credentials_from_file(&path)?;
            return Ok((creds, CredentialsSource::File(path)));
        }
    }

    anyhow::bail!(
        "No credentials found. Set {} / {} or place {} in one of: {:?}",
        ENV_ACCESS_KEY,
        ENV_SECRET_KEY,
        DEFAULT_CREDENTIALS_FILE,
        credentials_file_candidates()
    )
}

fn credentials_file_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = env::var(ENV_CREDENTIALS_FILE) {
        candidates.push(PathBuf::from(path));
    }

    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join(DEFAULT_CREDENTIALS_FILE));
    }

    if let Some(project_dirs) = ProjectDirs::from("com", "hcforge", "hc-forge") {
        candidates.push(project_dirs.config_dir().join(DEFAULT_CREDENTIALS_FILE));
    }

    if let Some(user_dirs) = UserDirs::new() {
        candidates.push(
            user_dirs
                .home_dir()
                .join(".huaweicloud")
                .join(DEFAULT_CREDENTIALS_FILE),
        );
    }

    candidates
}

fn load_credentials_from_file(path: &Path) -> Result<Credentials> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read credentials file: {}", path.display()))?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(contents.as_bytes());

    for record in reader.records() {
        let record = record.context("Failed to parse credentials CSV")?;
        if record.len() < 2 {
            continue;
        }

        let access_key = record.get(0).unwrap_or("").trim();
        let secret_key = record.get(1).unwrap_or("").trim();

        if access_key.is_empty() || secret_key.is_empty() {
            continue;
        }

        if looks_like_header(access_key) && looks_like_header(secret_key) {
            continue;
        }

        return Ok(Credentials::new(
            access_key.to_string(),
            secret_key.to_string(),
        ));
    }

    anyhow::bail!(
        "No usable credentials found in file: {}",
        path.display()
    )
}

fn looks_like_header(value: &str) -> bool {
    let value = value.to_lowercase();
    value.contains("access")
        || value.contains("secret")
        || value.contains("key id")
        || value == "ak"
        || value == "sk"
}
