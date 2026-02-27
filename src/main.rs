use std::{error::Error, sync::Arc};

use axum::Router;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, normalize_path::NormalizePathLayer, trace::TraceLayer,
};
use tracing::info;

use crate::{config::Config, state::AppState};

mod config;
mod state;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().unwrap_or_default();
    let config: Arc<Config> = Arc::new(Config::new());
    tracing_subscriber::fmt::init();

    info!("Starting server");

    info!("Creating state");
    let state: Arc<AppState> = Arc::new(AppState::new());

    info!("Initializing axum");
    let router: Router<()> = Router::new()
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(NormalizePathLayer::append_trailing_slash()),
        )
        .with_state(state.clone());

    info!("Initializing HTTP listener");
    let listener: TcpListener = TcpListener::bind(config.host.clone())
        .await
        .expect("HOST should be bindable");

    info!("Server is listening on {}", config.host);
    axum::serve(listener, router).await.unwrap_or_default();

    Ok(())
}
