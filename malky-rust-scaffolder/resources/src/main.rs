use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "__APP_NAME__=debug,tower_http=debug".parse().unwrap()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = __APP_NAME__::config::Config::from_env().expect("Failed to load config");

    let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = __APP_NAME__::state::AppState::new(pool);
    let app = __APP_NAME__::router::create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.api_port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
