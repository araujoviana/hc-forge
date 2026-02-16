/// Module for Huawei Cloud credentials used by backend API calls.

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
    Explicit,
}
