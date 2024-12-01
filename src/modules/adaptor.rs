use axum::http::HeaderMap;
use crate::spec::PreconfRequestParams;
use alloy::hex;
use chrono::Utc;

#[derive(Debug)]
pub struct AdaptedResult {
    pub headers: HeaderMap,
    pub body: serde_json::Value,
    pub url: String,
}

pub fn handle_adapter(params: PreconfRequestParams, genesis_time: u64) -> AdaptedResult {
    match params.proposer.source.as_str() {
        "bolt" => adapt_bolt(&params),
        "interstate" => adapt_interstate(&params),
        "primev" => adapt_primev(&params, genesis_time),
        _ => panic!("Unsupported source"),
    }
}

pub fn adapt_bolt(params: &PreconfRequestParams) -> AdaptedResult {
    let mut headers= HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json".parse().unwrap(),
    );
    let sidecar_signature = format!("0x{}", hex::encode(params.sidecar_signature.as_bytes()));   
    headers.insert(
        "X-Bolt-Signature",
        format!("{}:{}", params.signer, sidecar_signature).parse().unwrap(),
    );

    let signed_tx = format!("0x{}", hex::encode(params.signed_tx.envelope_encoded()));
    let body = serde_json::json!({
        "id": "1",
        "jsonrpc": "2.0",
        "method": "bolt_requestInclusion",
        "params": [{ "slot": params.proposer.slot, "txs": [signed_tx] }]
    });

    AdaptedResult {
        headers,
        body,
        url: params.proposer.sidecar_url.clone(),
    }
}

pub fn adapt_primev(params: &PreconfRequestParams, genesis_time: u64) -> AdaptedResult {
    let holesky_genesis_time = genesis_time;
    let decay_end_timestamp = (holesky_genesis_time + params.proposer.slot * 12) * 1000;
    let decay_start_timestamp = Utc::now().timestamp_millis();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json".parse().unwrap(),
    );
    let mut data = Vec::new();
    params.signed_tx.encode_enveloped(&mut data);
    let tx = format!("0x{}", hex::encode(&data));

    let body = serde_json::json!({
        "rawTransactions": [tx],
        "amount": "100000040",
        "blockNumber": params.proposer.slot,
        "decayStartTimestamp": decay_start_timestamp,
        "decayEndTimestamp": decay_end_timestamp,
        "revertingTxHashes": []
    });

    tracing::debug!("body: {:#?}", body);

    AdaptedResult {
        headers,
        body,
        url: params.proposer.sidecar_url.clone(),
    }
}

pub fn adapt_interstate(params: &PreconfRequestParams) -> AdaptedResult {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json".parse().unwrap(),
    );

    let sidecar_signature = format!("0x{}", hex::encode(params.sidecar_signature.as_bytes()));   
    let signed_tx = format!("0x{}", hex::encode(params.signed_tx.envelope_encoded()));
    
    let body = serde_json::json!({
        "id": "1",
        "jsonrpc": "2.0",
        "method": "submit_inclusion_preconfirmation",
        "messages": [{ "slot": params.proposer.slot, "tx": signed_tx, "signature": sidecar_signature }]
    });

    AdaptedResult {
        headers,
        body,
        url: params.proposer.sidecar_url.clone(),
    }
}
