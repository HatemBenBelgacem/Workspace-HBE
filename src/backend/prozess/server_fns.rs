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
        "SELECT s.id, s.process_id, s.position, s.activity, s.role, s.input, s.output, s.condition,
                s.system, s.parent_step_id, s.branch, s.is_start, s.is_end, s.element, s.task_type,
                s.event_type, s.loop_type, s.gateway, s.start_event, s.start_label, s.end_event,
                s.end_label, s.called_process_id, p.name AS called_process_name
         FROM process_steps s
         LEFT JOIN processes p ON p.id = s.called_process_id
         WHERE s.process_id = $1 ORDER BY s.position, s.id",
    )
    .bind(process_id)
    .fetch_all(&pool)
    .await
    .map_err(db::err)
}

/// Prüft die Felder eines Schritts vor dem Speichern.
#[cfg(feature = "ssr")]
fn checked(fields: StepFields, process_id: i64) -> Result<StepFields, ServerFnError> {
    db::required(&fields.activity, "Bezeichnung")?;
    let fields = fields.normalized().map_err(ServerFnError::new)?;
    if fields.called_process_id == Some(process_id) {
        return Err(ServerFnError::new("Ein Teilprozess kann nicht den eigenen Prozess aufrufen"));
    }
    Ok(fields)
}

/// Fügt einen Schritt am Ende des Hauptablaufs oder eines Zweigs an.
#[server]
pub async fn add_step(
    process_id: i64,
    parent_step_id: Option<i64>,
    branch: Option<String>,
    fields: StepFields,
) -> Result<(), ServerFnError> {
    let fields = checked(fields, process_id)?;
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
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO process_steps (process_id, parent_step_id, branch, position, activity)
         VALUES ($1, $2, $3,
                 (SELECT COALESCE(MAX(position), 0) + 1 FROM process_steps
                  WHERE process_id = $1
                    AND parent_step_id IS NOT DISTINCT FROM $2
                    AND branch IS NOT DISTINCT FROM $3),
                 $4)
         RETURNING id",
    )
    .bind(process_id)
    .bind(parent_step_id)
    .bind(branch)
    .bind(&fields.activity)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::err)?;
    write_fields(&mut tx, id, &fields).await?;
    tx.commit().await.map_err(db::err)?;
    Ok(())
}

#[server]
pub async fn update_step(id: i64, fields: StepFields) -> Result<(), ServerFnError> {
    let pool = db::pool()?;
    let mut tx = pool.begin().await.map_err(db::err)?;
    let (process_id, has_children): (i64, bool) = sqlx::query_as(
        "SELECT process_id,
                EXISTS (SELECT 1 FROM process_steps c WHERE c.parent_step_id = s.id)
         FROM process_steps s WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::err)?
    .ok_or_else(|| ServerFnError::new("Schritt nicht gefunden"))?;
    let fields = checked(fields, process_id)?;
    if fields.gateway.is_empty() && has_children {
        return Err(ServerFnError::new(
            "Die Verzweigung kann erst entfernt werden, wenn ihre Zweige keine Schritte mehr enthalten",
        ));
    }
    if fields.is_start {
        clear_start(&mut tx, process_id).await?;
    }
    write_fields(&mut tx, id, &fields).await?;
    tx.commit().await.map_err(db::err)?;
    Ok(())
}

/// Schreibt alle bearbeitbaren Felder eines Schritts.
#[cfg(feature = "ssr")]
async fn write_fields(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: i64,
    f: &StepFields,
) -> Result<(), ServerFnError> {
    sqlx::query(
        "UPDATE process_steps
         SET activity = $2, role = $3, input = $4, output = $5, condition = $6, system = $7,
             is_start = $8, is_end = $9, element = $10, task_type = $11, event_type = $12,
             loop_type = $13, gateway = $14, start_event = $15, start_label = $16,
             end_event = $17, end_label = $18, called_process_id = $19
         WHERE id = $1",
    )
    .bind(id)
    .bind(&f.activity)
    .bind(&f.role)
    .bind(&f.input)
    .bind(&f.output)
    .bind(&f.condition)
    .bind(&f.system)
    .bind(f.is_start)
    .bind(f.is_end)
    .bind(&f.element)
    .bind(&f.task_type)
    .bind(&f.event_type)
    .bind(&f.loop_type)
    .bind(&f.gateway)
    .bind(&f.start_event)
    .bind(&f.start_label)
    .bind(&f.end_event)
    .bind(&f.end_label)
    .bind(f.called_process_id)
    .execute(&mut **tx)
    .await
    .map_err(db::err)?;
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
