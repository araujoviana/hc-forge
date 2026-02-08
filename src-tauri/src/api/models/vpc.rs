use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Vpc {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct VpcListResponse {
    pub vpcs: Vec<Vpc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subnet {
    pub id: String,
    pub name: String,
    pub cidr: String,
}

#[derive(Debug, Deserialize)]
pub struct SubnetListResponse {
    pub subnets: Vec<Subnet>,
}

// Display formatting removed: it was only needed for CLI-based selection.
