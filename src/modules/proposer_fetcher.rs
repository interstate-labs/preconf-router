use crate::config::AppConfig;
use crate::spec::{Sidecar, BoltSidecar, InterstateSidecar, PrimevItems};
use reqwest::Client;
use std::time::Duration;
use tokio::time;
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::{debug, error, warn};

pub struct ProposerFetcher {
    config: AppConfig,
    client: Client,
    sidecars: Arc<Mutex<Vec<Sidecar>>>
}

impl ProposerFetcher {
    pub fn new(config: AppConfig, sidecars:Arc<Mutex<Vec<Sidecar>>>) -> Self {
        Self {
            config,
            client: Client::new(),
            sidecars
        }
    }

    // Runs the updater at specified intervals
    pub async fn run(&self, interval_seconds: u64) {
        let mut interval = time::interval(Duration::from_secs(interval_seconds));
        loop {
            interval.tick().await;
            if let Err(err) = self.update().await {
                error!("Failed to update sidecars: {:?}", err);
            }
        }
    }

    // Aggregates all sidecar data by calling each proposer-fetching method
    async fn update(&self) -> Result<(), reqwest::Error> {
        let mut updated_sidecars = Vec::new();

        updated_sidecars.extend(self.get_bolt_proposers().await?);
        updated_sidecars.extend(self.get_interstate_proposers().await?);
        updated_sidecars.extend(self.get_primev_proposers().await?);

        // Lock the mutex and update the shared sidecars list
        let mut sidecars = self.sidecars.lock().await;
        *sidecars = updated_sidecars;

        tracing::debug!("{:#?}", sidecars);
        Ok(())
    }

    // Fetch bolt proposers
    async fn get_bolt_proposers(&self) -> Result<Vec<Sidecar>, reqwest::Error> {
        if let Some(url) = &self.config.holesky_bolt_url {
            let request_url = format!("{}/proposers", url);
            debug!("sending request url:");
            debug!(request_url);
            match self
                .client
                .get(request_url)
                .send()
                .await
            {
                Ok(response) => {
                    let response_text = response.text().await?;
                    // debug!("Bolt response: {}", response_text);

                    let bolt_sidecars: Vec<BoltSidecar> = serde_json::from_str(&response_text).unwrap_or_default();
                    debug!("Got {} bolt proposers", bolt_sidecars.len());
                    debug!("This is the number of proposers in the current 32 slots. You can check if this is working properly by visiting: {}", &self.config.holesky_bolt_url);
        
                    Ok(bolt_sidecars.into_iter().map(|sidecar| Sidecar {
                        validator_index: sidecar.validator_index,
                        sidecar_url: sidecar.sidecar_url,
                        source: "bolt".to_string(),
                        slot: sidecar.slot,
                    }).collect())
                }
                Err(e) => {
                    warn!("{:#?}", e.to_string());
                    Ok(Vec::new())
                }
            }

           
        } else {
            warn!("Undefined BOLT gateway");
            Ok(Vec::new())
        }
    }

    // Fetch interstate proposers
    async fn get_interstate_proposers(&self) -> Result<Vec<Sidecar>, reqwest::Error> {
        if let Some(url) = &self.config.holesky_interstate_url {
            match self
                .client
                .get(format!("{}/proposers/lookahead?activeOnly=true&futureOnly=true", url))
                .send()
                .await
            {
                Ok(response) => {
                    let interstate_sidecars: Vec<InterstateSidecar> = response.json().await.unwrap_or_default();
                    debug!("Got {} interstate proposers", interstate_sidecars.len());
                    debug!("This is the number of proposers in the current 32 slots. You can check if this is working properly by visiting: {}", &self.config.holesky_interstate_url);
        
                    Ok(interstate_sidecars.into_iter().map(|sidecar| Sidecar {
                        validator_index: sidecar.validator_index,
                        sidecar_url: sidecar.sidecar_url,
                        source: "interstate".to_string(),
                        slot: sidecar.slot,
                    }).collect())
                }   
                Err(e) => {
                    warn!("{}", e.to_string());
                    Ok(Vec::new())
                }
            }
           
        } else {
            warn!("Undefined INTERSTATE gateway");
            Ok(Vec::new())
        }
    }

    // Fetch primev proposers
    async fn get_primev_proposers(&self) -> Result<Vec<Sidecar>, reqwest::Error> {
        if let Some(url) = &self.config.holesky_primev_url{
            match self
                .client
                .get(format!("{}/v1/validator/get_validators", url))
                .send()
                .await
                {
                    Ok(resp) => {
                        let primev_items: PrimevItems = resp.json().await.unwrap();
                        let mut primev_proposers = Vec::new();
            
                        for (slot, item) in primev_items.items {
                            if item.isOptedIn {
                                primev_proposers.push(Sidecar {
                                    slot: slot.parse().unwrap_or(0),
                                    validator_index: 0,
                                    sidecar_url: format!("{}/v1/bidder/bid", url),
                                    source: "primev".to_string(),
                                });
                            }
                        }
                        debug!("Got {} primev proposers", primev_proposers.len());
                        Ok(primev_proposers)
                    }
                    Err(e) => {
                        warn!("{:#?}", e.to_string());
                        Ok(Vec::new())
                    }
                }
        } else {
            warn!("Undefined PRIMEV client URL");
            Ok(Vec::new())
        }
    }
}
