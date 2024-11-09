use serde::Deserialize;
use config::ConfigError;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub holesky_bolt_gateway_url: Option<String>,
    pub holesky_interstate_gateway_url: Option<String>,
    pub holesky_beacon_url: Option<String>,
    pub holesky_rpc: Option<String>,
    pub holesky_genesis_time: Option<String>,
    pub holesky_primev_bid_client_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok(); // Load environment variables from `.env` file

        let config = config::Config::builder()
            .set_default("holesky_bolt_gateway_url", None::<String>)?
            .set_default("holesky_interstate_gateway_url", None::<String>)?
            .set_default("holesky_beacon_url", None::<String>)?
            .set_default("holesky_rpc", None::<String>)?
            .set_default("holesky_genesis_time", None::<String>)?
            .set_default("holesky_primev_bid_client_url", None::<String>)?
            .add_source(config::Environment::default()) // Load from environment
            .build()?;

        // Deserialize into `AppConfig`
        config.try_deserialize()
    }
}