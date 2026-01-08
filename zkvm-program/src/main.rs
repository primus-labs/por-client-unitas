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
const FEATURE_BALANCE_URL: &str = "https://fapi.binance.com/fapi/v3/balance";

const ASTER_SPOT_BALANCE_URL: &str = "https://sapi.asterdex.com/api/v1/account";
const ASTER_FEATURE_BALANCE_URL: &str = "https://fapi.asterdex.com/fapi/v2/balance";

const STABLE_COINS: &[&str] = &[
    "USDT", "USDC", "FDUSD", "TUSD", "USDE", "XUSD", "USD1", "BFUSD", "USDP", "DAI", "USDF",
];
const EPSILON_VALUE: f64 = 0.00000000001;

fn app_binance_spot(
    pv: &mut AttestationMetaStruct,
    attestation_data: &String,
    attestation_config: &AttestationConfig,
    asset_bals: &mut HashMap<String, f64>,
) -> Result<(), ZktlsError> {
    //
    // 1. Verify
    let mut attestation_config = attestation_config.clone();
    attestation_config.url = vec![SPOT_BALANCE_URL.to_string()];
    let attestation_config = serde_json::to_string(&attestation_config).unwrap();
    let (attestation_data, _, messages) = verify_attestation_data(&attestation_data, &attestation_config)
        .map_err(|e| zkerr!(ZkErrorCode::VerifyAttestation, e.to_string()))?;

    pv.task_id = attestation_data.public_data[0].taskId.clone();
    pv.report_tx_hash = attestation_data.public_data[0].reportTxHash.clone();
    pv.attestor = attestation_data.public_data[0].attestor.clone();
    pv.base_urls.push(SPOT_BALANCE_URL.to_string());

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

fn app_binance_future(
    pv: &mut AttestationMetaStruct,
    attestation_data: &String,
    attestation_config: &AttestationConfig,
    asset_bals: &mut HashMap<String, f64>,
) -> Result<(), ZktlsError> {
    //
    // 1. Verify
    let mut attestation_config = attestation_config.clone();
    attestation_config.url = vec![FEATURE_BALANCE_URL.to_string()];
    let attestation_config = serde_json::to_string(&attestation_config).unwrap();
    let (attestation_data, _, messages) = verify_attestation_data(&attestation_data, &attestation_config)
        .map_err(|e| zkerr!(ZkErrorCode::VerifyAttestation, e.to_string()))?;

    pv.task_id = attestation_data.public_data[0].taskId.clone();
    pv.report_tx_hash = attestation_data.public_data[0].reportTxHash.clone();
    pv.attestor = attestation_data.public_data[0].attestor.clone();
    pv.base_urls.push(FEATURE_BALANCE_URL.to_string());

    //
    // 2. Do some valid checks
    // In the vast majority of cases, it is legal. Data is extracted while the inspection is conducted.
    let msg_len = messages.len();
    let requests = attestation_data.public_data[0].attestation.request.clone();
    let requests_len = requests.len();
    ensure_zk!(requests_len == msg_len, zkerr!(ZkErrorCode::InvalidMessagesLength));

    let mut i = 0;
    let mut uid_paths = vec![];
    uid_paths.push("$.[*].accountAlias");

    let mut bal_paths = vec![];
    bal_paths.push("$.[*].asset");
    bal_paths.push("$.[*].balance");
    bal_paths.push("$.[*].crossUnPnl");

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
        if !request.url.starts_with(FEATURE_BALANCE_URL) {
            return Err(zkerr!(ZkErrorCode::InvalidRequestUrl));
        }

        {
            // uid
            let json_value = messages[i]
                .get_json_values(&uid_paths)
                .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;
            if json_value.len() == 0 {
                continue; // no any data of future response
            }

            ensure_zk!(json_value.len() > 0, zkerr!(ZkErrorCode::InvalidJsonValueSize));

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
                let bal: f64 = json_value[size + j].trim_matches('"').parse().unwrap_or(0.0);
                let un_pnl: f64 = json_value[size * 2 + j].trim_matches('"').parse().unwrap_or(0.0);
                *asset_bals.entry(asset.to_string()).or_insert(0.0) += bal + un_pnl;
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

fn app_binance_unified(
    pv: &mut AttestationMetaStruct,
    attestation_data: &String,
    attestation_config: &AttestationConfig,
    asset_bals: &mut HashMap<String, f64>,
) -> Result<(), ZktlsError> {
    //
    // 1. Verify
    let mut attestation_config = attestation_config.clone();
    attestation_config.url = vec![RISK_URL.to_string(), BALANCE_URL.to_string()];
    let attestation_config = serde_json::to_string(&attestation_config).unwrap();
    let (attestation_data, _, messages) = verify_attestation_data(&attestation_data, &attestation_config)
        .map_err(|e| zkerr!(ZkErrorCode::VerifyAttestation, e.to_string()))?;

    pv.task_id = attestation_data.public_data[0].taskId.clone();
    pv.report_tx_hash = attestation_data.public_data[0].reportTxHash.clone();
    pv.attestor = attestation_data.public_data[0].attestor.clone();
    pv.base_urls.push(RISK_URL.to_string());
    pv.base_urls.push(BALANCE_URL.to_string());

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
            if !um_price.is_empty() {
                um_prices.push(um_price);
            }
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

fn app_aster_spot(
    pv: &mut AttestationMetaStruct,
    attestation_data: &String,
    attestation_config: &AttestationConfig,
    asset_bals: &mut HashMap<String, f64>,
) -> Result<(), ZktlsError> {
    //
    // 1. Verify
    let mut attestation_config = attestation_config.clone();
    attestation_config.url = vec![ASTER_SPOT_BALANCE_URL.to_string()];
    let attestation_config = serde_json::to_string(&attestation_config).unwrap();
    let (attestation_data, _, messages) = verify_attestation_data(&attestation_data, &attestation_config)
        .map_err(|e| zkerr!(ZkErrorCode::VerifyAttestation, e.to_string()))?;

    pv.task_id = attestation_data.public_data[0].taskId.clone();
    pv.report_tx_hash = attestation_data.public_data[0].reportTxHash.clone();
    pv.attestor = attestation_data.public_data[0].attestor.clone();
    pv.base_urls.push(ASTER_SPOT_BALANCE_URL.to_string());
    //
    // 2. Do some valid checks
    // In the vast majority of cases, it is legal. Data is extracted while the inspection is conducted.
    let msg_len = messages.len();
    let requests = attestation_data.public_data[0].attestation.request.clone();
    let requests_len = requests.len();
    ensure_zk!(requests_len == msg_len, zkerr!(ZkErrorCode::InvalidMessagesLength));

    let mut i = 0;
    let mut uid_paths = vec![];
    uid_paths.push("$.updateTime");

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
        if !request.url.starts_with(ASTER_SPOT_BALANCE_URL) {
            return Err(zkerr!(ZkErrorCode::InvalidRequestUrl));
        }

        let update_time;
        {
            // uid
            let json_value = messages[i]
                .get_json_values(&uid_paths)
                .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;

            ensure_zk!(json_value.len() == 1, zkerr!(ZkErrorCode::InvalidJsonValueSize));

            update_time = json_value[0].trim_matches('"').to_string();
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

            let mut _uid = vec![];
            let size = json_value.len() / bal_paths.len();
            for j in 0..size {
                let asset = json_value[j].trim_matches('"').to_ascii_uppercase();
                let free: f64 = json_value[size + j].trim_matches('"').parse().unwrap_or(0.0);
                let locked: f64 = json_value[size * 2 + j].trim_matches('"').parse().unwrap_or(0.0);
                *asset_bals.entry(asset.to_string()).or_insert(0.0) += free + locked;

                // for uid check
                let v = format!("{}:{}:{}", asset, free, locked);
                _uid.push(v);
            }
            _uid.sort();
            let _uid = _uid.join(",");
            if !_uid.is_empty() {
                let _uid = format!("{}:{}", update_time, _uid);
                uids.push(_uid);
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

fn app_aster_future(
    pv: &mut AttestationMetaStruct,
    attestation_data: &String,
    attestation_config: &AttestationConfig,
    asset_bals: &mut HashMap<String, f64>,
) -> Result<(), ZktlsError> {
    //
    // 1. Verify
    let mut attestation_config = attestation_config.clone();
    attestation_config.url = vec![ASTER_FEATURE_BALANCE_URL.to_string()];
    let attestation_config = serde_json::to_string(&attestation_config).unwrap();
    let (attestation_data, _, messages) = verify_attestation_data(&attestation_data, &attestation_config)
        .map_err(|e| zkerr!(ZkErrorCode::VerifyAttestation, e.to_string()))?;

    pv.task_id = attestation_data.public_data[0].taskId.clone();
    pv.report_tx_hash = attestation_data.public_data[0].reportTxHash.clone();
    pv.attestor = attestation_data.public_data[0].attestor.clone();
    pv.base_urls.push(ASTER_FEATURE_BALANCE_URL.to_string());

    //
    // 2. Do some valid checks
    // In the vast majority of cases, it is legal. Data is extracted while the inspection is conducted.
    let msg_len = messages.len();
    let requests = attestation_data.public_data[0].attestation.request.clone();
    let requests_len = requests.len();
    ensure_zk!(requests_len == msg_len, zkerr!(ZkErrorCode::InvalidMessagesLength));

    let mut i = 0;
    let mut uid_paths = vec![];
    uid_paths.push("$.[*].accountAlias");

    let mut bal_paths = vec![];
    bal_paths.push("$.[*].asset");
    bal_paths.push("$.[*].balance");
    bal_paths.push("$.[*].crossUnPnl");

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
        if !request.url.starts_with(ASTER_FEATURE_BALANCE_URL) {
            return Err(zkerr!(ZkErrorCode::InvalidRequestUrl));
        }

        {
            // uid
            let json_value = messages[i]
                .get_json_values(&uid_paths)
                .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;
            if json_value.len() == 0 {
                continue; // no any data of future response
            }

            ensure_zk!(json_value.len() > 0, zkerr!(ZkErrorCode::InvalidJsonValueSize));

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
                let bal: f64 = json_value[size + j].trim_matches('"').parse().unwrap_or(0.0);
                let un_pnl: f64 = json_value[size * 2 + j].trim_matches('"').parse().unwrap_or(0.0);
                *asset_bals.entry(asset.to_string()).or_insert(0.0) += bal + un_pnl;
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

fn app_binance(
    pv: &mut PublicValuesStruct,
    attestations: &HashMap<String, String>,
    attestation_config: &AttestationConfig,
) -> Result<(), ZktlsError> {
    // Verify Unified, Spot and Future
    let mut asset_bals: HashMap<String, f64> = HashMap::new();

    if let Some(spot_data) = attestations.get("binanceSpot") {
        let mut spot_am = AttestationMetaStruct::default();
        app_binance_spot(&mut spot_am, &spot_data, attestation_config, &mut asset_bals)?;
        pv.attestation_meta.push(spot_am);
    }

    if let Some(future_data) = attestations.get("binanceUsdSFuture") {
        let mut future_am = AttestationMetaStruct::default();
        app_binance_future(&mut future_am, &future_data, attestation_config, &mut asset_bals)?;
        pv.attestation_meta.push(future_am);
    }

    if let Some(unified_data) = attestations.get("binanceUnified") {
        let mut unified_am = AttestationMetaStruct::default();
        app_binance_unified(&mut unified_am, &unified_data, attestation_config, &mut asset_bals)?;
        pv.attestation_meta.push(unified_am);
    }

    // Summary assets by Category
    let mut asset_balance: HashMap<String, f64> = HashMap::new();
    let mut stablecoin_sum = 0.0;
    for (k, v) in asset_bals {
        if STABLE_COINS.contains(&k.as_str()) {
            stablecoin_sum += v;
        } else {
            if v > EPSILON_VALUE {
                asset_balance.insert(k, v);
            }
        }
    }
    if stablecoin_sum > EPSILON_VALUE {
        asset_balance.insert("STABLECOIN".to_string(), stablecoin_sum);
    }
    pv.asset_balance.insert("binance".to_string(), asset_balance);

    Ok(())
}

fn app_aster(
    pv: &mut PublicValuesStruct,
    attestations: &HashMap<String, String>,
    attestation_config: &AttestationConfig,
) -> Result<(), ZktlsError> {
    // Verify Spot and Future
    let mut asset_bals: HashMap<String, f64> = HashMap::new();

    if let Some(spot_data) = attestations.get("asterSpot") {
        let mut spot_am = AttestationMetaStruct::default();
        app_aster_spot(&mut spot_am, &spot_data, attestation_config, &mut asset_bals)?;
        pv.attestation_meta.push(spot_am);
    }
    if let Some(future_data) = attestations.get("asterFuture") {
        let mut future_am = AttestationMetaStruct::default();
        app_aster_future(&mut future_am, &future_data, attestation_config, &mut asset_bals)?;
        pv.attestation_meta.push(future_am);
    }

    // Summary assets by Category
    let mut asset_balance: HashMap<String, f64> = HashMap::new();
    let mut stablecoin_sum = 0.0;
    for (k, v) in asset_bals {
        if STABLE_COINS.contains(&k.as_str()) {
            stablecoin_sum += v;
        } else {
            if v > EPSILON_VALUE {
                asset_balance.insert(k, v);
            }
        }
    }
    if stablecoin_sum > EPSILON_VALUE {
        asset_balance.insert("STABLECOIN".to_string(), stablecoin_sum);
    }
    pv.asset_balance.insert("aster".to_string(), asset_balance);

    Ok(())
}

/// Helper function
fn set_meta(pv: &mut PublicValuesStruct, attestations: &HashMap<String, String>) -> Result<(), ZktlsError> {
    if let Some(meta) = attestations.get("__meta__") {
        let meta: HashMap<String, String> =
            serde_json::from_str(meta).map_err(|e| zkerr!(ZkErrorCode::ParseMetaData, e.to_string()))?;

        pv.project_id = meta
            .get("projectId")
            .ok_or_else(|| zkerr!(ZkErrorCode::MissingProjectId))?
            .to_owned();
    }

    Ok(())
}

fn app_main(pv: &mut PublicValuesStruct) -> Result<(), ZktlsError> {
    let config_data: String = sp1_zkvm::io::read();
    let attestations: HashMap<String, String> = sp1_zkvm::io::read();

    set_meta(pv, &attestations)?;

    let attestation_config: AttestationConfig =
        serde_json::from_str(&config_data).map_err(|e| zkerr!(ZkErrorCode::ParseConfigData, e.to_string()))?;

    app_binance(pv, &attestations, &attestation_config)?;
    app_aster(pv, &attestations, &attestation_config)?;

    Ok(())
}

pub fn main() {
    let mut pv = PublicValuesStruct::default();
    pv.kind = "asset-balance".to_string();
    pv.version = "0.1.0".to_string();
    if let Err(e) = app_main(&mut pv) {
        println!("Error: {} {}", e.icode(), e.msg());
        pv.status = e.icode();
    } else {
        println!("OK");
    }
    commit(&pv);
}
