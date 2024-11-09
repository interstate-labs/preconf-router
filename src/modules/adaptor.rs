use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposer {
    pub source: String,
    pub slot: u64,
    pub sidecar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreconfRequestParams {
    pub proposer: Proposer,
    pub signer: String,
    pub signature: String,
    pub signed_tx: String,
}

#[derive(Debug)]
pub struct AdaptedResult {
    pub headers: HeaderMap,
    pub body: serde_json::Value,
    pub url: String,
}

pub fn handle_adapter(params: PreconfRequestParams, genesis_time: u64) -> AdaptedResult {
    match params.proposer.source.as_str() {
        "bolt" => adapt_bolt(&params),
        "interstate" => adapt_interstate(),
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
    headers.insert(
        "X-Bolt-Signature",
        format!("{}:{}", params.signer, params.signature).parse().unwrap(),
    );

    let body = serde_json::json!({
        "id": "1",
        "jsonrpc": "2.0",
        "method": "bolt_requestInclusion",
        "params": [{ "slot": params.proposer.slot, "txs": [params.signed_tx] }]
    });

    AdaptedResult {
        headers,
        body,
        url: params.proposer.sidecar_url.clone(),
    }
}

pub fn adapt_primev(params: &PreconfRequestParams, genesis_time: u64) -> AdaptedResult {
    let holesky_genesis_time = genesis_time;
    let decay_start_timestamp = (holesky_genesis_time + params.proposer.slot * 12) * 1000;
    let decay_end_timestamp = decay_start_timestamp + 500;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json".parse().unwrap(),
    );

    let body = serde_json::json!({
        "rawTransactions": [params.signed_tx],
        "amount": "100000040",
        "blockNumber": params.proposer.slot,
        "decayStartTimestamp": decay_start_timestamp,
        "decayEndTimestamp": decay_end_timestamp,
        "revertingTxHashes": []
    });

    AdaptedResult {
        headers,
        body,
        url: params.proposer.sidecar_url.clone(),
    }
}

pub fn adapt_interstate() -> AdaptedResult {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json".parse().unwrap(),
    );

    let body = serde_json::json!({
        "id": "1"
    });

    AdaptedResult {
        headers,
        body,
        url: "".to_string(),
    }
}

// pub fn adapt_luban() -> AdaptedResult {}

// pub fn adapt_ethgas() -> AdaptedResult {}
