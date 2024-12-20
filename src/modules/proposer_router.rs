// src/utils.rs
use crate::config::AppConfig;
use crate::spec::Sidecar;
use reqwest::Client;
use reqwest::Error as ReqwestError;
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::{debug, error, warn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("internal server error: `{0}`")]
    InternalServerError(u32),

    #[error("Request failed")]
    RequestError(#[from] ReqwestError), // Wraps reqwest errors
}
#[derive(Clone)]
pub struct ProposerRouter {
    pub client: Client,
    pub config: AppConfig,
    pub sidecars: Arc<Mutex<Vec<Sidecar>>>, // Updated to tokio::sync::Mutex
}
impl ProposerRouter {
    pub fn new(config: AppConfig, sidecars: Arc<Mutex<Vec<Sidecar>>>) -> Self {
        Self {
            client: Client::new(),
            config,
            sidecars,
        }
    }

    pub async fn find_appropriate_proposer(&self) -> Option<Sidecar> {
      let slot = self.get_latest_slot().await.unwrap_or(0);
      debug!("Current slot: {}", slot);

      // Use the async lock
      let sidecars = self.sidecars.lock().await;

      let mut next_proposers: Vec<_> = sidecars
          .iter()
          .filter(|sidecar| sidecar.slot > slot)
          .cloned()
          .collect();
      next_proposers.sort_by_key(|sidecar| sidecar.slot);

      for proposer in next_proposers {
          if self.check_health(&proposer.sidecar_url).await.unwrap_or(false) {
              return Some(proposer);
          }
      }

      None
  }
    // Fetch the latest slot
    async fn get_latest_slot(&self) -> Result<u64, FetchError> {
        if let Some(ref url) = self.config.holesky_beacon_url {
            let response = self
                .client
                .get(format!("{}/eth/v1/beacon/headers/head", url))
                .send()
                .await
                .map_err(FetchError::RequestError)?;

            let data: serde_json::Value = response.json().await.map_err(FetchError::RequestError)?;
            
            if let Some(slot) = data["data"]["header"]["message"]["slot"].as_str() {
                return slot.parse().map_err(|_| FetchError::InternalServerError(500));
            }
        } else {
            warn!("Undefined HOLESKY_BEACON_URL");
        }
        Ok(0)
    }

    // Check the health of a sidecar by making a GET request to its URL
    async fn check_health(&self, sidecar_url: &str) -> Result<bool, reqwest::Error> {
        match self.client.get(sidecar_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(true)
                } else if matches!(response.status().as_u16(), 400 | 404 | 405 | 501) {
                    Ok(true)
                } else {
                    debug!("{}", response.status().as_u16());
                    Ok(false)
                }
            }
            Err(err) => {
                error!("Error checking health of {}: {:?}", sidecar_url, err);
                Ok(false)
            }
        }
    }
}
