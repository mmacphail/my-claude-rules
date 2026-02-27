use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Pagination ---

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl PaginationParams {
    pub fn page(&self) -> i64 { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).clamp(1, 1000) }
    pub fn offset(&self) -> i64 { (self.page() - 1) * self.per_page() }
}

// --- Models ---

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// --- Request bodies ---

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemRequest {
    pub name: String,
}

// --- List response ---

#[derive(Debug, Serialize)]
pub struct Meta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub meta: Meta,
}

impl<T> ListResponse<T> {
    pub fn new(data: Vec<T>, total: i64, params: &PaginationParams) -> Self {
        Self {
            meta: Meta { total, page: params.page(), per_page: params.per_page() },
            data,
        }
    }

    pub fn from_full(data: Vec<T>) -> Self {
        let total = data.len() as i64;
        Self {
            meta: Meta { total, page: 1, per_page: total.max(1) },
            data,
        }
    }
}
