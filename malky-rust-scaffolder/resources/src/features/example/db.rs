use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use super::model::Item;

pub async fn count_all(pool: &PgPool) -> Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM items WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn find_all(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Item>> {
    let rows = sqlx::query_as::<_, Item>(
        "SELECT id, name, created_at, updated_at, deleted_at
         FROM items
         WHERE deleted_at IS NULL
         ORDER BY created_at ASC
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Item> {
    let row = sqlx::query_as::<_, Item>(
        "SELECT id, name, created_at, updated_at, deleted_at
         FROM items
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(row)
}

pub async fn insert(pool: &PgPool, name: &str) -> Result<Item> {
    let row = sqlx::query_as::<_, Item>(
        "INSERT INTO items (id, name, created_at, updated_at)
         VALUES (gen_random_uuid(), $1, NOW(), NOW())
         RETURNING id, name, created_at, updated_at, deleted_at",
    )
    .bind(name)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update(pool: &PgPool, id: Uuid, name: &str) -> Result<Item> {
    let row = sqlx::query_as::<_, Item>(
        "UPDATE items
         SET name = $2, updated_at = NOW()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, name, created_at, updated_at, deleted_at",
    )
    .bind(id)
    .bind(name)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(row)
}

pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let result = sqlx::query(
        "UPDATE items SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
