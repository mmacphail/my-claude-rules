use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .merge(crate::features::example::router());

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}
