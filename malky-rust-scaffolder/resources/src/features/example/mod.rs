pub mod db;
pub mod handlers;
pub mod model;

use axum::{routing::get, Router};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/items", get(handlers::list_items).post(handlers::create_item))
        .route(
            "/items/{id}",
            get(handlers::get_item)
                .patch(handlers::update_item)
                .delete(handlers::delete_item),
        )
}
