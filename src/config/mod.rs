use serde::Deserialize;
use config::ConfigError;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    // Holesky
    pub holesky_bolt_url: Option<String>,
    pub holesky_primev_url: Option<String>, 
    pub holesky_luban_url: Option<String>,
    pub holesky_interstate_url: Option<String>,
    pub holesky_beacon_url: Option<String>,
    pub holesky_rpc: Option<String>,
    pub holesky_genesis_time: Option<String>,
    
    // Mainnet
    pub mainnet_interstate_url: Option<String>,

    // Other
    pub port: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok(); // Load environment variables from `.env` file

        let config = config::Config::builder()
            // Holesky
            .set_default("holesky_bolt_url", None::<String>)?
            .set_default("holesky_primev_url", None::<String>)?
            .set_default("holesky_luban_url", None::<String>)?
            .set_default("holesky_interstate_url", None::<String>)?
            .set_default("holesky_beacon_url", None::<String>)?

            .set_default("holesky_rpc", None::<String>)?
            .set_default("holesky_genesis_time", None::<String>)?
            
            // Mainnet
            .set_default("mainnet_interstate_url", None::<String>)?
            
            // Other
            .set_default("port", None::<String>)?

            // Load from environment
            .add_source(config::Environment::default()) 
            .build()?;

        // Deserialize into `AppConfig`
        config.try_deserialize()
    }
}