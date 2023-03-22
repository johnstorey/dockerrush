use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Blob {
    pub digest: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub media_type: String,
    pub config: Blob,
    pub layers: Vec<Blob>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub tags: Vec<String>,
}
