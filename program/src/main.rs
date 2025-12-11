// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use anyhow::Result;
use sp1_zkvm::io::commit;
use std::collections::{HashMap, HashSet};
use zktls_att_verification::attestation_data::verify_attestation_data;
use zktls_att_verification::attestation_data::AttestationConfig;

mod errors;
use errors::{ZkErrorCode, ZktlsError};
mod structs;
use structs::{AttestationMetaStruct, PublicValuesStruct};

const RISK_URL: &str = "https://papi.binance.com/papi/v1/um/positionRisk";
const BALANCE_URL: &str = "https://papi.binance.com/papi/v1/balance";
const SPOT_BALANCE_URL: &str = "https://api.binance.com/api/v3/account";
const UNIFIED_URL: &[&str] = &[RISK_URL, BALANCE_URL];
const SPOT_URL: &[&str] = &[SPOT_BALANCE_URL];
const STABLE_COINS: &[&str] = &[
    "USDT", "USDC", "FDUSD", "TUSD", "USDE", "XUSD", "USD1", "BFUSD", "USDP", "DAI",
];

fn app_unified(
    pv: &mut AttestationMetaStruct,
    attestation_data: &String,
    attestation_config: &AttestationConfig,
    asset_bals: &mut HashMap<String, f64>,
) -> Result<(), ZktlsError> {
    //
    // 0. Make attestation config
    let v: serde_json::Value = serde_json::from_str(&attestation_data)
        .map_err(|e| zkerr!(ZkErrorCode::ParseAttestationData, e.to_string()))?;
    let task_id = v
        .get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("taskId"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetTaskIdFail))?;
    let report_tx_hash = v
        .get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("reportTxHash"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetReportTxHashFail))?;
    let attestor_addr = v
        .get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("attestor"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetAttestorAddressFail))?;
    pv.task_id = task_id.to_string();
    pv.report_tx_hash = report_tx_hash.to_string();
    pv.attestor = attestor_addr.to_string();
    pv.base_urls.push(RISK_URL.to_string());
    pv.base_urls.push(BALANCE_URL.to_string());

    //
    // 1. Verify
    let mut attestation_config = attestation_config.clone();
    attestation_config.url = UNIFIED_URL.iter().map(|s| s.to_string()).collect();
    let attestation_config = serde_json::to_string(&attestation_config).unwrap();
    let (attestation_data, _, messages) = verify_attestation_data(&attestation_data, &attestation_config)
        .map_err(|e| zkerr!(ZkErrorCode::VerifyAttestation, e.to_string()))?;

    //
    // 2. Do some valid checks
    // In the vast majority of cases, it is legal. Data is extracted while the inspection is conducted.
    let msg_len = messages.len();
    let requests = attestation_data.public_data[0].attestation.request.clone();
    let requests_len = requests.len();
    ensure_zk!(requests_len % 2 == 0, zkerr!(ZkErrorCode::InvalidRequestLength));
    ensure_zk!(requests_len == msg_len, zkerr!(ZkErrorCode::InvalidMessagesLength));

    let mut i = 0;
    let mut um_paths = vec![];
    um_paths.push("$.[*].symbol");
    um_paths.push("$.[*].entryPrice");

    let mut bal_paths = vec![];
    bal_paths.push("$.[*].asset");
    bal_paths.push("$.[*].totalWalletBalance");
    bal_paths.push("$.[*].umUnrealizedPNL");

    pv.timestamp = u128::MAX;
    let mut um_prices = vec![];
    // strict order: um1 bal1 um2 bal2 ...
    for request in requests {
        let ts = request
            .url
            .split("timestamp=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .filter(|s| !s.is_empty())
            .ok_or(zkerr!(ZkErrorCode::CannotFoundTimestamp))?
            .parse::<u128>()
            .map_err(|_| zkerr!(ZkErrorCode::ParseTimestampFailed))?;
        pv.timestamp = pv.timestamp.min(ts);

        // check url and get assets' balance
        if request.url.starts_with(RISK_URL) {
            ensure_zk!(i % 2 == 0, zkerr!(ZkErrorCode::InvalidRequestOrder));

            let json_value = messages[i]
                .get_json_values(&um_paths)
                .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;

            ensure_zk!(
                json_value.len() % um_paths.len() == 0,
                zkerr!(ZkErrorCode::InvalidJsonValueSize)
            );

            // Collects UM (asset => entryPrice) info
            let mut prices = vec![];
            let size = json_value.len() / um_paths.len();
            for j in 0..size {
                let asset = json_value[j].trim_matches('"').to_ascii_uppercase();
                let price = json_value[size + j].trim_matches('"').to_string();
                let v = format!("{}:{}", asset, price);
                prices.push(v);
            }
            prices.sort();
            let um_price = prices.join(",");
            um_prices.push(um_price);
        } else if request.url.starts_with(BALANCE_URL) {
            let json_value = messages[i]
                .get_json_values(&bal_paths)
                .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;

            ensure_zk!(
                json_value.len() % bal_paths.len() == 0,
                zkerr!(ZkErrorCode::InvalidJsonValueSize)
            );

            let size = json_value.len() / bal_paths.len();
            for j in 0..size {
                let asset = json_value[j].trim_matches('"').to_ascii_uppercase();
                let bal: f64 = json_value[size + j].trim_matches('"').parse().unwrap_or(0.0);
                let pnl: f64 = json_value[size * 2 + j].trim_matches('"').parse().unwrap_or(0.0);
                *asset_bals.entry(asset.to_string()).or_insert(0.0) += bal + pnl;
            }
        } else {
            return Err(zkerr!(ZkErrorCode::InvalidRequestUrl));
        }

        i += 1;
    }

    // Is the account duplicate?
    let mut seen = HashSet::new();
    ensure_zk!(
        !um_prices.iter().any(|x| !seen.insert(x)),
        zkerr!(ZkErrorCode::DuplicateAccount)
    );

    Ok(())
}

fn app_spot(
    pv: &mut AttestationMetaStruct,
    attestation_data: &String,
    attestation_config: &AttestationConfig,
    asset_bals: &mut HashMap<String, f64>,
) -> Result<(), ZktlsError> {
    //
    // 0. Make attestation config
    let v: serde_json::Value = serde_json::from_str(&attestation_data)
        .map_err(|e| zkerr!(ZkErrorCode::ParseAttestationData, e.to_string()))?;
    let task_id = v
        .get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("taskId"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetTaskIdFail))?;
    let report_tx_hash = v
        .get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("reportTxHash"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetReportTxHashFail))?;
    let attestor_addr = v
        .get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("attestor"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetAttestorAddressFail))?;
    pv.task_id = task_id.to_string();
    pv.report_tx_hash = report_tx_hash.to_string();
    pv.attestor = attestor_addr.to_string();
    pv.base_urls.push(SPOT_BALANCE_URL.to_string());

    //
    // 1. Verify
    let mut attestation_config = attestation_config.clone();
    attestation_config.url = SPOT_URL.iter().map(|s| s.to_string()).collect();
    let attestation_config = serde_json::to_string(&attestation_config).unwrap();
    let (attestation_data, _, messages) = verify_attestation_data(&attestation_data, &attestation_config)
        .map_err(|e| zkerr!(ZkErrorCode::VerifyAttestation, e.to_string()))?;

    //
    // 2. Do some valid checks
    // In the vast majority of cases, it is legal. Data is extracted while the inspection is conducted.
    let msg_len = messages.len();
    let requests = attestation_data.public_data[0].attestation.request.clone();
    let requests_len = requests.len();
    ensure_zk!(requests_len == msg_len, zkerr!(ZkErrorCode::InvalidMessagesLength));

    let mut i = 0;
    let mut uid_paths = vec![];
    uid_paths.push("$.uid");

    let mut bal_paths = vec![];
    bal_paths.push("$.balances[*].asset");
    bal_paths.push("$.balances[*].free");
    bal_paths.push("$.balances[*].locked");

    pv.timestamp = u128::MAX;
    let mut uids = vec![];
    for request in requests {
        let ts = request
            .url
            .split("timestamp=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .filter(|s| !s.is_empty())
            .ok_or(zkerr!(ZkErrorCode::CannotFoundTimestamp))?
            .parse::<u128>()
            .map_err(|_| zkerr!(ZkErrorCode::ParseTimestampFailed))?;
        pv.timestamp = pv.timestamp.min(ts);

        // check url
        if !request.url.starts_with(SPOT_BALANCE_URL) {
            return Err(zkerr!(ZkErrorCode::InvalidRequestUrl));
        }

        {
            // uid
            let json_value = messages[i]
                .get_json_values(&uid_paths)
                .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;

            ensure_zk!(json_value.len() == 1, zkerr!(ZkErrorCode::InvalidJsonValueSize));

            let uid = json_value[0].trim_matches('"').to_string();
            uids.push(uid);
        }

        {
            // balance
            let json_value = messages[i]
                .get_json_values(&bal_paths)
                .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;

            ensure_zk!(
                json_value.len() % bal_paths.len() == 0,
                zkerr!(ZkErrorCode::InvalidJsonValueSize)
            );

            let size = json_value.len() / bal_paths.len();
            for j in 0..size {
                let asset = json_value[j].trim_matches('"').to_ascii_uppercase();
                let free: f64 = json_value[size + j].trim_matches('"').parse().unwrap_or(0.0);
                let locked: f64 = json_value[size * 2 + j].trim_matches('"').parse().unwrap_or(0.0);
                *asset_bals.entry(asset.to_string()).or_insert(0.0) += free + locked;
            }
        }

        i += 1;
    }

    // Is the account duplicate?
    let mut seen = HashSet::new();
    ensure_zk!(
        !uids.iter().any(|x| !seen.insert(x)),
        zkerr!(ZkErrorCode::DuplicateAccount)
    );

    Ok(())
}
fn app_main(pv: &mut PublicValuesStruct) -> Result<(), ZktlsError> {
    let attestation_data: String = sp1_zkvm::io::read();
    let config_data: String = sp1_zkvm::io::read();
    let attestation_config: AttestationConfig =
        serde_json::from_str(&config_data).map_err(|e| zkerr!(ZkErrorCode::ParseConfigData, e.to_string()))?;

    // Get Unified and Spot data
    let v: serde_json::Value = serde_json::from_str(&attestation_data)
        .map_err(|e| zkerr!(ZkErrorCode::ParseAttestationData, e.to_string()))?;
    let unified_data = v
        .get("unified")
        .map(|a| a.to_string())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetUnifiedDataFail))?;

    let spot_data = v
        .get("spot")
        .map(|a| a.to_string())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetSpotDataFail))?;

    // Verify Unified and Spot
    let mut asset_bals: HashMap<String, f64> = HashMap::new();

    let mut unified_am = AttestationMetaStruct::default();
    app_unified(&mut unified_am, &unified_data, &attestation_config, &mut asset_bals)?;
    pv.attestation_meta.push(unified_am);

    let mut spot_am = AttestationMetaStruct::default();
    app_spot(&mut spot_am, &spot_data, &attestation_config, &mut asset_bals)?;
    pv.attestation_meta.push(spot_am);

    // Summary assets by Category
    let mut stablecoin_sum = 0.0;
    for (k, v) in asset_bals {
        if STABLE_COINS.contains(&k.as_str()) {
            stablecoin_sum += v;
        } else {
            pv.asset_balance.insert(k, v);
        }
    }
    pv.asset_balance.insert("STABLECOIN".to_string(), stablecoin_sum);

    Ok(())
}

pub fn main() {
    let mut pv = PublicValuesStruct::default();
    if let Err(e) = app_main(&mut pv) {
        println!("Error: {} {}", e.icode(), e.msg());
        pv.status = e.icode();
    } else {
        println!("OK");
    }
    commit(&pv);
}
