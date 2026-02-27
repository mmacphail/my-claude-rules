use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{error::Result, state::AppState};
use super::{
    db,
    model::{CreateItemRequest, ListResponse, PaginationParams, UpdateItemRequest},
};

pub async fn list_items(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<ListResponse<super::model::Item>>> {
    let total = db::count_all(&state.db).await?;
    let items = db::find_all(&state.db, pagination.per_page(), pagination.offset()).await?;
    Ok(Json(ListResponse::new(items, total, &pagination)))
}

pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<super::model::Item>> {
    let item = db::find_by_id(&state.db, id).await?;
    Ok(Json(item))
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(body): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<super::model::Item>)> {
    if body.name.trim().is_empty() {
        return Err(crate::error::AppError::BadRequest("name is required".into()));
    }
    if body.name.len() > 255 {
        return Err(crate::error::AppError::BadRequest("name must be 255 characters or fewer".into()));
    }
    let item = db::insert(&state.db, &body.name).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateItemRequest>,
) -> Result<Json<super::model::Item>> {
    if body.name.trim().is_empty() {
        return Err(crate::error::AppError::BadRequest("name is required".into()));
    }
    let item = db::update(&state.db, id, &body.name).await?;
    Ok(Json(item))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    db::soft_delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
