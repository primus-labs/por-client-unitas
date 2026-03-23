use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct AttestationMetaStruct {
    pub task_id: String,
    pub report_tx_hash: String,
    pub attestor: String,
    pub base_urls: Vec<String>,
    pub timestamp: u128,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct PublicValuesStruct {
    pub kind: String,
    pub version: String,
    pub project_id: String,
    pub attestation_meta: Vec<AttestationMetaStruct>,
    pub asset_balance: HashMap<String, HashMap<String, f64>>, // source => { asset => balance }
    pub status: i16,
}
