use leptos::prelude::*;

use super::model::{Entity, EntityColumn};
#[cfg(feature = "ssr")]
use crate::backend::db;

#[server]
pub async fn list_entities() -> Result<Vec<Entity>, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as(
        "SELECT e.id, e.name, COUNT(c.id) AS column_count
         FROM entities e LEFT JOIN entity_columns c ON c.entity_id = e.id
         GROUP BY e.id ORDER BY e.name",
    )
    .fetch_all(&pool)
    .await
    .map_err(db::err)
}

#[server]
pub async fn get_entity(id: i64) -> Result<Entity, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as(
        "SELECT e.id, e.name, (SELECT COUNT(*) FROM entity_columns WHERE entity_id = e.id) AS column_count
         FROM entities e WHERE e.id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(db::err)?
    .ok_or_else(|| ServerFnError::new("Entität nicht gefunden"))
}

#[server]
pub async fn create_entity(name: String) -> Result<i64, ServerFnError> {
    db::required(&name, "Bezeichnung")?;
    let pool = db::pool()?;
    sqlx::query_scalar("INSERT INTO entities (name) VALUES ($1) RETURNING id")
        .bind(name.trim())
        .fetch_one(&pool)
        .await
        .map_err(db::err)
}

#[server]
pub async fn rename_entity(id: i64, name: String) -> Result<(), ServerFnError> {
    db::required(&name, "Bezeichnung")?;
    let pool = db::pool()?;
    sqlx::query("UPDATE entities SET name = $2 WHERE id = $1")
        .bind(id)
        .bind(name.trim())
        .execute(&pool)
        .await
        .map_err(db::err)?;
    Ok(())
}

#[server]
pub async fn delete_entity(id: i64) -> Result<(), ServerFnError> {
    let pool = db::pool()?;
    sqlx::query("DELETE FROM entities WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db::err)?;
    Ok(())
}

#[server]
pub async fn list_entity_columns(entity_id: i64) -> Result<Vec<EntityColumn>, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as(
        "SELECT id, entity_id, name, data_type, meaning, references_entity_id
         FROM entity_columns WHERE entity_id = $1 ORDER BY position, id",
    )
    .bind(entity_id)
    .fetch_all(&pool)
    .await
    .map_err(db::err)
}

/// Alle Spalten aller Entitäten, für das ER-Diagramm.
#[server]
pub async fn list_all_entity_columns() -> Result<Vec<EntityColumn>, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as(
        "SELECT id, entity_id, name, data_type, meaning, references_entity_id
         FROM entity_columns ORDER BY entity_id, position, id",
    )
    .fetch_all(&pool)
    .await
    .map_err(db::err)
}

#[server]
pub async fn add_entity_column(
    entity_id: i64,
    name: String,
    data_type: String,
    meaning: String,
    references_entity_id: Option<i64>,
) -> Result<(), ServerFnError> {
    db::required(&name, "Bezeichnung")?;
    let pool = db::pool()?;
    sqlx::query(
        "INSERT INTO entity_columns (entity_id, position, name, data_type, meaning, references_entity_id)
         VALUES ($1, (SELECT COALESCE(MAX(position), 0) + 1 FROM entity_columns WHERE entity_id = $1),
                 $2, $3, $4, $5)",
    )
    .bind(entity_id)
    .bind(name.trim())
    .bind(data_type.trim())
    .bind(meaning.trim())
    .bind(references_entity_id)
    .execute(&pool)
    .await
    .map_err(db::err)?;
    Ok(())
}

#[server]
pub async fn update_entity_column(
    id: i64,
    name: String,
    data_type: String,
    meaning: String,
    references_entity_id: Option<i64>,
) -> Result<(), ServerFnError> {
    db::required(&name, "Bezeichnung")?;
    let pool = db::pool()?;
    sqlx::query(
        "UPDATE entity_columns
         SET name = $2, data_type = $3, meaning = $4, references_entity_id = $5
         WHERE id = $1",
    )
    .bind(id)
    .bind(name.trim())
    .bind(data_type.trim())
    .bind(meaning.trim())
    .bind(references_entity_id)
    .execute(&pool)
    .await
    .map_err(db::err)?;
    Ok(())
}

#[server]
pub async fn delete_entity_column(id: i64) -> Result<(), ServerFnError> {
    let pool = db::pool()?;
    sqlx::query("DELETE FROM entity_columns WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db::err)?;
    Ok(())
}
