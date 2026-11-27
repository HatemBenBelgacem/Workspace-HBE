use leptos::prelude::*;

use super::model::{Process, ProcessStep, StepFields};
#[cfg(feature = "ssr")]
use crate::backend::db;

#[server]
pub async fn list_processes() -> Result<Vec<Process>, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as("SELECT id, name, description FROM processes ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(db::err)
}

#[server]
pub async fn get_process(id: i64) -> Result<Process, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as("SELECT id, name, description FROM processes WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(db::err)?
        .ok_or_else(|| ServerFnError::new("Prozess nicht gefunden"))
}

#[server]
pub async fn create_process(name: String, description: String) -> Result<i64, ServerFnError> {
    db::required(&name, "Name")?;
    let pool = db::pool()?;
    sqlx::query_scalar("INSERT INTO processes (name, description) VALUES ($1, $2) RETURNING id")
        .bind(name.trim())
        .bind(description.trim())
        .fetch_one(&pool)
        .await
        .map_err(db::err)
}

#[server]
pub async fn delete_process(id: i64) -> Result<(), ServerFnError> {
    let pool = db::pool()?;
    sqlx::query("DELETE FROM processes WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db::err)?;
    Ok(())
}

#[server]
pub async fn list_steps(process_id: i64) -> Result<Vec<ProcessStep>, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as(
        "SELECT id, process_id, position, activity, role, input, output, condition, system,
                parent_step_id, branch, is_start, is_end
         FROM process_steps WHERE process_id = $1 ORDER BY position, id",
    )
    .bind(process_id)
    .fetch_all(&pool)
    .await
    .map_err(db::err)
}

/// Fügt einen Schritt am Ende des Hauptablaufs oder eines Ja-/Nein-Zweigs an.
#[server]
pub async fn add_step(
    process_id: i64,
    parent_step_id: Option<i64>,
    branch: Option<String>,
    fields: StepFields,
) -> Result<(), ServerFnError> {
    db::required(&fields.activity, "Aktivität")?;
    if parent_step_id.is_some() != branch.is_some()
        || branch.as_deref().is_some_and(|b| b != "Ja" && b != "Nein")
    {
        return Err(ServerFnError::new("Ungültiger Zweig"));
    }
    let pool = db::pool()?;
    let mut tx = pool.begin().await.map_err(db::err)?;
    if fields.is_start {
        clear_start(&mut tx, process_id).await?;
    }
    sqlx::query(
        "INSERT INTO process_steps
             (process_id, parent_step_id, branch, position, activity, role, input, output, condition, system,
              is_start, is_end)
         VALUES ($1, $2, $3,
                 (SELECT COALESCE(MAX(position), 0) + 1 FROM process_steps
                  WHERE process_id = $1
                    AND parent_step_id IS NOT DISTINCT FROM $2
                    AND branch IS NOT DISTINCT FROM $3),
                 $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(process_id)
    .bind(parent_step_id)
    .bind(branch)
    .bind(fields.activity.trim())
    .bind(fields.role.trim())
    .bind(fields.input.trim())
    .bind(fields.output.trim())
    .bind(fields.condition.trim())
    .bind(fields.system.trim())
    .bind(fields.is_start)
    .bind(fields.is_end)
    .execute(&mut *tx)
    .await
    .map_err(db::err)?;
    tx.commit().await.map_err(db::err)?;
    Ok(())
}

/// Entfernt den bisherigen Startpunkt, damit ein anderer Schritt Startpunkt werden kann.
#[cfg(feature = "ssr")]
async fn clear_start(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    process_id: i64,
) -> Result<(), ServerFnError> {
    sqlx::query("UPDATE process_steps SET is_start = false WHERE process_id = $1 AND is_start")
        .bind(process_id)
        .execute(&mut **tx)
        .await
        .map_err(db::err)?;
    Ok(())
}

#[server]
pub async fn update_step(id: i64, fields: StepFields) -> Result<(), ServerFnError> {
    db::required(&fields.activity, "Aktivität")?;
    let pool = db::pool()?;
    let mut tx = pool.begin().await.map_err(db::err)?;
    if fields.is_start {
        let process_id: i64 = sqlx::query_scalar("SELECT process_id FROM process_steps WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db::err)?
            .ok_or_else(|| ServerFnError::new("Schritt nicht gefunden"))?;
        clear_start(&mut tx, process_id).await?;
    }
    sqlx::query(
        "UPDATE process_steps
         SET activity = $2, role = $3, input = $4, output = $5, condition = $6, system = $7,
             is_start = $8, is_end = $9
         WHERE id = $1",
    )
    .bind(id)
    .bind(fields.activity.trim())
    .bind(fields.role.trim())
    .bind(fields.input.trim())
    .bind(fields.output.trim())
    .bind(fields.condition.trim())
    .bind(fields.system.trim())
    .bind(fields.is_start)
    .bind(fields.is_end)
    .execute(&mut *tx)
    .await
    .map_err(db::err)?;
    tx.commit().await.map_err(db::err)?;
    Ok(())
}

#[server]
pub async fn delete_step(id: i64) -> Result<(), ServerFnError> {
    let pool = db::pool()?;
    sqlx::query("DELETE FROM process_steps WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db::err)?;
    Ok(())
}

/// Tauscht die Position eines Schritts mit seinem Vorgänger (`up`) oder Nachfolger.
#[server]
pub async fn move_step(id: i64, up: bool) -> Result<(), ServerFnError> {
    let pool = db::pool()?;
    let mut tx = pool.begin().await.map_err(db::err)?;

    let (process_id, position, parent_step_id, branch): (i64, i32, Option<i64>, Option<String>) =
        sqlx::query_as(
            "SELECT process_id, position, parent_step_id, branch FROM process_steps WHERE id = $1",
        )
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(db::err)?;

    let neighbor_sql = if up {
        "SELECT id, position FROM process_steps
         WHERE process_id = $1 AND position < $2
           AND parent_step_id IS NOT DISTINCT FROM $3 AND branch IS NOT DISTINCT FROM $4
         ORDER BY position DESC LIMIT 1"
    } else {
        "SELECT id, position FROM process_steps
         WHERE process_id = $1 AND position > $2
           AND parent_step_id IS NOT DISTINCT FROM $3 AND branch IS NOT DISTINCT FROM $4
         ORDER BY position ASC LIMIT 1"
    };
    let neighbor: Option<(i64, i32)> = sqlx::query_as(neighbor_sql)
        .bind(process_id)
        .bind(position)
        .bind(parent_step_id)
        .bind(branch)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::err)?;

    if let Some((neighbor_id, neighbor_position)) = neighbor {
        for (step_id, new_position) in [(id, neighbor_position), (neighbor_id, position)] {
            sqlx::query("UPDATE process_steps SET position = $2 WHERE id = $1")
                .bind(step_id)
                .bind(new_position)
                .execute(&mut *tx)
                .await
                .map_err(db::err)?;
        }
    }

    tx.commit().await.map_err(db::err)?;
    Ok(())
}
