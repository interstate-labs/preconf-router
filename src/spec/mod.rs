use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize)]
pub struct BoltSidecar {
    pub slot: u64,
    pub validator_index: u32,
    pub validator_pubkey: String,
    pub sidecar_url: String,
}

#[derive(Debug, Deserialize)]
pub struct InterstateSidecar {
    pub validator_index: u32,
    pub sidecar_url: String,
    pub slot: u64,
}

#[derive(Debug, Deserialize)]
pub struct PrimevItems {
    pub items: std::collections::HashMap<String, PrimevItem>,
}

#[derive(Debug, Deserialize)]
pub struct PrimevItem {
    pub isOptedIn: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sidecar {
    pub validator_index: u32,
    pub sidecar_url: String,
    pub source: String,
    pub slot: u64,
}