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
use axum::extract::State;
use axum::response::Html;
use askama::Template;


use modules::{proposer_fetcher::ProposerFetcher,proposer_router::ProposerRouter};

use config::AppConfig;

#[derive(Template)]
#[template(path = "holesky.html")]
struct ProposerStatsTemplateHolesky {
    bolt_proposers: Vec<Sidecar>,
    interstate_proposers: Vec<Sidecar>,
    current_slot: String,
}

#[derive(Template)]
#[template(path = "mainnet.html")]
struct ProposerStatsTemplateMainnet {
    bolt_proposers: Vec<Sidecar>,
    interstate_proposers: Vec<Sidecar>,
    current_slot: String,
}

async fn stats_handler_holesky(State(proposer_router): State<Arc<ProposerRouter>>) -> Html<String> {
    let sidecars = proposer_router.as_ref().get_sidecars().await;
    
    // Split and sort proposers by source
    let mut bolt_proposers: Vec<_> = sidecars.iter()
        .filter(|s| s.source == "bolt")
        .cloned()
        .collect();
    bolt_proposers.sort_by_key(|s| s.slot);

    let mut interstate_proposers: Vec<_> = sidecars.iter()
        .filter(|s| s.source == "interstate")
        .cloned()
        .collect();
    interstate_proposers.sort_by_key(|s| s.slot);
    
    // Add current time
    let current_slot = proposer_router.get_latest_slot().await.unwrap_or(0).to_string();

    let template = ProposerStatsTemplateHolesky {
        bolt_proposers,
        interstate_proposers,
        current_slot,
    };
    
    Html(template.render().unwrap_or_else(|_| String::from("Error rendering template")))
}


async fn stats_handler_mainnet(State(proposer_router): State<Arc<ProposerRouter>>) -> Html<String> {
    let sidecars = proposer_router.as_ref().get_sidecars().await;
    
    // Split and sort proposers by source
    let mut bolt_proposers: Vec<_> = sidecars.iter()
        .filter(|s| s.source == "bolt")
        .cloned()
        .collect();
    bolt_proposers.sort_by_key(|s| s.slot);

    let mut interstate_proposers: Vec<_> = sidecars.iter()
        .filter(|s| s.source == "interstate")
        .cloned()
        .collect();
    interstate_proposers.sort_by_key(|s| s.slot);
    
    // Add current time
    let current_slot = proposer_router.get_latest_slot().await.unwrap_or(0).to_string();

    let template = ProposerStatsTemplateMainnet {
        bolt_proposers,
        interstate_proposers,
        current_slot,
    };
    
    Html(template.render().unwrap_or_else(|_| String::from("Error rendering template")))
}

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
    let port = config.port.clone()  // Clone the Option<String> first
        .expect("Failed to parse port number")
        .parse::<u16>()
        .expect("Failed to parse port number");

    let fetcher = ProposerFetcher::new(config.clone(), Arc::clone(&sidecars));
    let proposer_router: Arc<ProposerRouter> = Arc::new(ProposerRouter::new(config, Arc::clone(&sidecars)));

 let app = Router::new()
    .route("/api/v1/proposer", get(find_proposer_handler))
    .route("/api/v1/submit", post(submit_preconfirmation))
    // Add this new route
    .route("/", get(stats_handler_holesky))
    .route("/holesky", get(stats_handler_holesky))
    .route("/mainnet", get(stats_handler_mainnet))
    .with_state(proposer_router);
    
    // Run ProposerFetcher in a separate task
    tokio::spawn(async move {
        fetcher.run(12).await; // Run with a 30-second interval
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}
