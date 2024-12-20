use axum::{debug_handler, extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::sync::Arc;
use reqwest::{Client, Error as ReqwestError};
use crate::modules::validator::ValidatedBody;
use crate::ProposerRouter;
use crate::modules::adaptor::handle_adapter;
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

#[debug_handler]
pub async fn submit_preconfirmation(
    State(proposer_router): State<Arc<ProposerRouter>>,
    ValidatedBody(body): ValidatedBody,
) -> Result<Json<SubmitResponse>, HandlerError>{
    
    let client = Client::new();
    let genesis_time:u64 = proposer_router.config.holesky_genesis_time.as_ref().unwrap().parse().expect("Failed to convert u64");
    // Handle adapter to get URL, body, and headers
    let adapted =  handle_adapter(body, genesis_time);

    let response = client.post(adapted.url)
    .json(&adapted.body)
    .headers(adapted.headers)
    .send()
    .await
    .map_err(|e| {
        tracing::error!(?e, "failed to submit preconfirmation");
        HandlerError::UnexpectedError("Unexpected Error in submitting preconf".to_string())
    })?;

    let response = response.text().await?;

    let response = response.replace(&"0".repeat(32), ".").replace(&".".repeat(4), "");
    tracing::info!("Response: {:?}", response);
    Ok(Json(SubmitResponse{message:true}))
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
