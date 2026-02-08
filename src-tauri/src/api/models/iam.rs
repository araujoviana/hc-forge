use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectsResponse {
    pub projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}
