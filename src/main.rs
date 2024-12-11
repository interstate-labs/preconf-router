use axum::{ routing::{get, post}, Router };
use handlers::{find_proposer_handler, submit_preconfirmation};
use spec::Sidecar;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing_subscriber::fmt::Subscriber;
mod config;
mod handlers;
mod spec;
mod modules;

use modules::{proposer_fetcher::ProposerFetcher,proposer_router::ProposerRouter};

use config::AppConfig;

#[tokio::main]
async fn main() {

    let subscriber = Subscriber::builder()
    .with_max_level(tracing::Level::DEBUG)
    .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
    tracing::debug!("Debug level logging initialized");

    // Shared sidecars list
    let sidecars = Arc::new(Mutex::new(Vec::<Sidecar>::new()));

    // Load config and create fetcher,router instance
    let config = AppConfig::from_env().expect("Failed to load configuration from environment");
    let fetcher = ProposerFetcher::new(config.clone(), Arc::clone(&sidecars));
    let proposer_router: Arc<ProposerRouter> = Arc::new(ProposerRouter::new(config, Arc::clone(&sidecars)));

 let app = Router::new()
    .route("/api/v1/proposer", get(find_proposer_handler))
    .route("/api/v1/submit", post(submit_preconfirmation))
    .with_state(proposer_router);
    
    // Run ProposerFetcher in a separate task
    tokio::spawn(async move {
        fetcher.run(12).await; // Run with a 30-second interval
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}
