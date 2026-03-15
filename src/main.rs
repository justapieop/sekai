mod config;
mod middleware;
mod repo;
mod routes;
mod state;
mod utils;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use sqlx::{migrate, postgres::PgPoolOptions, PgPool};
use std::{error::Error, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::repo::comment::CommentRepo;
use crate::utils::ai::AiUtils;
use crate::utils::signature::Signature;
use crate::{
    config::Config,
    repo::{
        challenge::ChallengeRepo, file::FileRepo, pin::PinRepo, pin_types::PinTypeRepo,
        post::PostRepo, user::UserRepo,
    },
    state::AppState,
    utils::{jwt_utils::JwtUtils, snowflake::SnowflakeGenerator, storage::StorageUtils},
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

    info!("Connecting to S3");
    let storage_utils: Arc<StorageUtils> = Arc::new(StorageUtils::new(
        &config.s3_endpoint,
        &config.s3_region,
        &config.s3_access_key_id,
        &config.s3_secret_access_key,
        &config.s3_bucket_name,
    ));

    info!("Connecting to database");
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("DATABASE_URL must be connected");

    info!("Verifying webhook secret");
    let signature_utils: Arc<Signature> =
        Arc::from(Signature::new(&config.authgear_webhook_secret));

    info!("Checking Gemini API key");
    let ai_utils: Arc<AiUtils> = Arc::new(AiUtils::new(&config.gemini_api_key));

    info!("Performing migration if needed");
    migrate!().run(&pool).await.unwrap_or_default();

    let user_repo: Arc<UserRepo> = Arc::new(UserRepo::new());
    let post_repo: Arc<PostRepo> = Arc::new(PostRepo::new());
    let file_repo: Arc<Mutex<FileRepo>> = Arc::new(Mutex::new(FileRepo::new()));
    let pin_repo: Arc<PinRepo> = Arc::new(PinRepo::new());
    let pin_type_repo: Arc<PinTypeRepo> = Arc::new(PinTypeRepo::new());
    let challenge_repo: Arc<ChallengeRepo> = Arc::new(ChallengeRepo::new());
    let comment_repo: Arc<CommentRepo> = Arc::new(CommentRepo::new());

    info!("Creating state");
    let state: Arc<AppState> = Arc::new(AppState::new(
        snowflake,
        jwt_utils,
        pool,
        storage_utils,
        user_repo,
        post_repo,
        file_repo,
        pin_repo,
        pin_type_repo,
        challenge_repo,
        signature_utils,
        comment_repo,
        ai_utils,
    ));

    info!("Initializing axum");
    let router: Router<()> = Router::new()
        .without_v07_checks()
        .merge(routes::routes(state.clone()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(NormalizePathLayer::trim_trailing_slash())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .layer(DefaultBodyLimit::max(5 * 1024 * 1024 * 1024)),
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
