use axum::{debug_handler, extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use reqwest::{Client, Error as ReqwestError};
use crate::ProposerRouter;
use crate::modules::adaptor::{handle_adapter, PreconfRequestParams};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("unexpected error: `{0}`")]
    UnexpectedError(String),
    #[error("Request failed")]
    RequestError(#[from] ReqwestError), // Wraps reqwest errors
}

#[derive(Serialize)]
pub struct ProposerResponse {
    slot: u64,
    validator_index: u32,
    sidecar_url: String,
    source: String,
}


pub async fn find_proposer_handler(
    State(proposer_router): State<Arc<ProposerRouter>>,
) -> Result<Json<ProposerResponse>, HandlerError> {
    if let Some(proposer) = proposer_router.find_appropriate_proposer().await {
        Ok(Json(ProposerResponse {
            slot: proposer.slot,
            validator_index: proposer.validator_index,
            sidecar_url: proposer.sidecar_url.clone(),
            source: proposer.source.clone(),
        }))
    } else {
        Err(HandlerError::UnexpectedError(
            "No appropriate proposer found".to_string(),
        ))
    }
}

#[derive(Deserialize)]
pub struct Proposer {
    pub slot: u64,
    pub validator_index: u64,
    pub sidecar_url: String,
    pub source: String,
}

#[debug_handler]
pub async fn submit_preconfirmation(
    State(proposer_router): State<Arc<ProposerRouter>>,
    Json(request_params): Json<PreconfRequestParams>,
) -> Result<Json<SubmitResponse>, HandlerError>{
    // validate request params

    // Strategies: 1) submit to first available 2) submit to all 3) submit to specific provider
    // if no imput is provided, default to first available

    let client = Client::new();
    let genesis_time:u64 = proposer_router.config.holesky_genesis_time.as_ref().unwrap().parse().expect("Failed to convert u64");
    // Handle adapter to get URL, body, and headers
    let adapted =  handle_adapter(request_params, genesis_time);

    // Send preconfirmation request
    match client.post(adapted.url)
        .json(&adapted.body)
        .headers(adapted.headers)
        .send()
        .await
    {
        Ok(_) => {
            tracing::info!("Preconfirmation sent successfully");
            Ok(Json(SubmitResponse{message:true}))
        },
        Err(_e) => {
           Err(HandlerError::UnexpectedError("Unexpected Error in submitting preconf".to_string()))
        },
    }
}

#[derive(Serialize)]
pub struct SubmitResponse {
    message: bool,
}


impl axum::response::IntoResponse for HandlerError {
    fn into_response(self) -> axum::response::Response {
       match self {
        HandlerError::RequestError(err) => {
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
        }
        HandlerError::UnexpectedError(err) => {
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
        }
       }
    }
}
