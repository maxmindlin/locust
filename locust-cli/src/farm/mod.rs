pub mod gcp;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreatedVM {
    pub name: String,
    pub zone: String,
    pub status: String,
    pub external_ip: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExistingVM {
    pub name: String,
}
