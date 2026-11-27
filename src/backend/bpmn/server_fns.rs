use leptos::prelude::*;

use super::model::{BpmnDiagram, BpmnDiagramContent};
#[cfg(feature = "ssr")]
use super::model::{empty_diagram, looks_like_bpmn};
#[cfg(feature = "ssr")]
use crate::backend::db;

#[server]
pub async fn list_diagrams() -> Result<Vec<BpmnDiagram>, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as(
        "SELECT id, name, description, to_char(updated_at AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS updated
         FROM bpmn_diagrams ORDER BY updated_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(db::err)
}

#[server]
pub async fn get_diagram(id: i64) -> Result<BpmnDiagramContent, ServerFnError> {
    let pool = db::pool()?;
    sqlx::query_as("SELECT id, name, description, xml FROM bpmn_diagrams WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(db::err)?
        .ok_or_else(|| ServerFnError::new("Diagramm nicht gefunden"))
}

#[server]
pub async fn create_diagram(name: String, description: String) -> Result<i64, ServerFnError> {
    db::required(&name, "Name")?;
    let pool = db::pool()?;
    sqlx::query_scalar("INSERT INTO bpmn_diagrams (name, description, xml) VALUES ($1, $2, $3) RETURNING id")
        .bind(name.trim())
        .bind(description.trim())
        .bind(empty_diagram(name.trim()))
        .fetch_one(&pool)
        .await
        .map_err(db::err)
}

/// Ändert Name und Beschreibung; der Inhalt bleibt unverändert.
#[server]
pub async fn update_diagram(id: i64, name: String, description: String) -> Result<(), ServerFnError> {
    db::required(&name, "Name")?;
    let pool = db::pool()?;
    sqlx::query("UPDATE bpmn_diagrams SET name = $2, description = $3, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(name.trim())
        .bind(description.trim())
        .execute(&pool)
        .await
        .map_err(db::err)?;
    Ok(())
}

/// Speichert den Inhalt aus dem Editor.
#[server]
pub async fn save_diagram_xml(id: i64, xml: String) -> Result<(), ServerFnError> {
    if !looks_like_bpmn(&xml) {
        return Err(ServerFnError::new("Der Inhalt ist kein BPMN-2.0-Dokument"));
    }
    let pool = db::pool()?;
    let result = sqlx::query("UPDATE bpmn_diagrams SET xml = $2, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(xml)
        .execute(&pool)
        .await
        .map_err(db::err)?;
    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Diagramm nicht gefunden"));
    }
    Ok(())
}

#[server]
pub async fn delete_diagram(id: i64) -> Result<(), ServerFnError> {
    let pool = db::pool()?;
    sqlx::query("DELETE FROM bpmn_diagrams WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db::err)?;
    Ok(())
}
