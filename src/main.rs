mod config;
mod middleware;
mod routes;
mod state;
mod utils;

use std::{error::Error, sync::Arc};

use axum::Router;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{net::TcpListener, sync::Mutex};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, normalize_path::NormalizePathLayer, trace::TraceLayer,
};
use tracing::info;

use crate::{
    config::Config,
    state::AppState,
    utils::{jwt_utils::JwtUtils, snowflake::SnowflakeGenerator},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().unwrap_or_default();
    let config: Arc<Config> = Arc::new(Config::new());
    tracing_subscriber::fmt::init();

    info!("Starting server");

    info!("Creating snowflake generator");
    let snowflake: Arc<Mutex<SnowflakeGenerator>> =
        Arc::new(Mutex::new(SnowflakeGenerator::new(config.machine_id)));

    info!("Verifying JWKS");
    let jwt_utils: Arc<JwtUtils> =
        Arc::new(JwtUtils::new(&config.jwks_iss, &config.jwks_url).await);

    info!("Connecting to database");
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("DATABASE_URL must be connected");

    info!("Creating state");
    let state: Arc<AppState> = Arc::new(AppState::new(snowflake, jwt_utils, pool));

    info!("Initializing axum");
    let router: Router<()> = Router::new()
        .without_v07_checks()
        .merge(routes::routes(state.clone()))
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
