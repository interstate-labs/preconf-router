use axum::{
    response::{Html, IntoResponse, Redirect}, // Add these
    routing::{get, get_service, post},
    Router,
    ServiceExt,
};
use handlers::{find_proposer_handler, submit_preconfirmation};
use reqwest::StatusCode;
use spec::Sidecar;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{fs, path::Path}; // Add this line
use tokio::sync::Mutex;
use tracing_subscriber::fmt::Subscriber;
mod config;
mod handlers;
mod modules;
mod spec;

use modules::{proposer_fetcher::ProposerFetcher, proposer_router::ProposerRouter};
use tower_http::services::ServeDir;

use config::AppConfig;

#[tokio::main]
async fn main() {
    let subscriber = Subscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::debug!("Debug level logging initialized");

    // Shared sidecars list
    let sidecars = Arc::new(Mutex::new(Vec::<Sidecar>::new()));

    // Load config and create fetcher,router instance
    let config = AppConfig::from_env().expect("Failed to load configuration from environment");
    let fetcher = ProposerFetcher::new(config.clone(), Arc::clone(&sidecars));
    let proposer_router: Arc<ProposerRouter> =
        Arc::new(ProposerRouter::new(config, Arc::clone(&sidecars)));

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/mainnet", get(mainnet_handler))
        .route("/holesky", get(holesky_handler))
        .route("/api/v1/proposer", get(find_proposer_handler))
        .route("/api/v1/submit", post(submit_preconfirmation))
        .with_state(proposer_router);

    async fn root_handler() -> impl IntoResponse {
        axum::response::Redirect::to("/holesky")
    }

    async fn mainnet_handler() -> impl IntoResponse {
        let content = fs::read_to_string("templates/mainnet.html")
            .map(Html)
            .unwrap_or_else(|_| Html("Error loading mainnet template".to_string()));
        content
    }

    async fn holesky_handler() -> impl IntoResponse {
        let content = fs::read_to_string("templates/holesky.html")
            .map(Html)
            .unwrap_or_else(|_| Html("Error loading holesky template".to_string()));
        content
    }

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
