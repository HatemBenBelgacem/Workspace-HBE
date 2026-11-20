use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Process {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProcessStep {
    pub id: i64,
    pub process_id: i64,
    pub position: i32,
    pub activity: String,
    pub role: String,
    pub input: String,
    pub output: String,
    pub condition: String,
    pub system: String,
    /// Schritt mit der Bedingung, zu dessen Ja- oder Nein-Zweig dieser Schritt gehört.
    pub parent_step_id: Option<i64>,
    /// "Ja" oder "Nein"; leer bei Schritten des Hauptablaufs.
    pub branch: Option<String>,
}

impl ProcessStep {
    pub fn fields(&self) -> StepFields {
        StepFields {
            activity: self.activity.clone(),
            role: self.role.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            condition: self.condition.clone(),
            system: self.system.clone(),
        }
    }
}

/// Die in der Tabelle bearbeitbaren Felder eines Schritts.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StepFields {
    pub activity: String,
    pub role: String,
    pub input: String,
    pub output: String,
    pub condition: String,
    pub system: String,
}

#[cfg(feature = "ssr")]
mod ssr {
    use leptos::prelude::*;
    use sqlx::PgPool;

    pub fn db() -> Result<PgPool, ServerFnError> {
        use_context::<PgPool>().ok_or_else(|| ServerFnError::new("Keine Datenbankverbindung"))
    }

    pub fn err(e: sqlx::Error) -> ServerFnError {
        ServerFnError::new(format!("Datenbankfehler: {e}"))
    }

    pub fn required(value: &str, field: &str) -> Result<(), ServerFnError> {
        if value.trim().is_empty() {
            Err(ServerFnError::new(format!("{field} darf nicht leer sein")))
        } else {
            Ok(())
        }
    }
}

#[server]
pub async fn list_processes() -> Result<Vec<Process>, ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query_as("SELECT id, name, description FROM processes ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(ssr::err)
}

#[server]
pub async fn get_process(id: i64) -> Result<Process, ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query_as("SELECT id, name, description FROM processes WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(ssr::err)?
        .ok_or_else(|| ServerFnError::new("Prozess nicht gefunden"))
}

#[server]
pub async fn create_process(name: String, description: String) -> Result<i64, ServerFnError> {
    ssr::required(&name, "Name")?;
    let pool = ssr::db()?;
    sqlx::query_scalar("INSERT INTO processes (name, description) VALUES ($1, $2) RETURNING id")
        .bind(name.trim())
        .bind(description.trim())
        .fetch_one(&pool)
        .await
        .map_err(ssr::err)
}

#[server]
pub async fn delete_process(id: i64) -> Result<(), ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query("DELETE FROM processes WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(ssr::err)?;
    Ok(())
}

#[server]
pub async fn list_steps(process_id: i64) -> Result<Vec<ProcessStep>, ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query_as(
        "SELECT id, process_id, position, activity, role, input, output, condition, system,
                parent_step_id, branch
         FROM process_steps WHERE process_id = $1 ORDER BY position, id",
    )
    .bind(process_id)
    .fetch_all(&pool)
    .await
    .map_err(ssr::err)
}

/// Fügt einen Schritt am Ende des Hauptablaufs oder eines Ja-/Nein-Zweigs an.
#[server]
pub async fn add_step(
    process_id: i64,
    parent_step_id: Option<i64>,
    branch: Option<String>,
    fields: StepFields,
) -> Result<(), ServerFnError> {
    ssr::required(&fields.activity, "Aktivität")?;
    if parent_step_id.is_some() != branch.is_some()
        || branch.as_deref().is_some_and(|b| b != "Ja" && b != "Nein")
    {
        return Err(ServerFnError::new("Ungültiger Zweig"));
    }
    let pool = ssr::db()?;
    sqlx::query(
        "INSERT INTO process_steps
             (process_id, parent_step_id, branch, position, activity, role, input, output, condition, system)
         VALUES ($1, $2, $3,
                 (SELECT COALESCE(MAX(position), 0) + 1 FROM process_steps
                  WHERE process_id = $1
                    AND parent_step_id IS NOT DISTINCT FROM $2
                    AND branch IS NOT DISTINCT FROM $3),
                 $4, $5, $6, $7, $8, $9)",
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
    .execute(&pool)
    .await
    .map_err(ssr::err)?;
    Ok(())
}

#[server]
pub async fn update_step(id: i64, fields: StepFields) -> Result<(), ServerFnError> {
    ssr::required(&fields.activity, "Aktivität")?;
    let pool = ssr::db()?;
    sqlx::query(
        "UPDATE process_steps
         SET activity = $2, role = $3, input = $4, output = $5, condition = $6, system = $7
         WHERE id = $1",
    )
    .bind(id)
    .bind(fields.activity.trim())
    .bind(fields.role.trim())
    .bind(fields.input.trim())
    .bind(fields.output.trim())
    .bind(fields.condition.trim())
    .bind(fields.system.trim())
    .execute(&pool)
    .await
    .map_err(ssr::err)?;
    Ok(())
}

#[server]
pub async fn delete_step(id: i64) -> Result<(), ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query("DELETE FROM process_steps WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(ssr::err)?;
    Ok(())
}

/// Tauscht die Position eines Schritts mit seinem Vorgänger (`up`) oder Nachfolger.
#[server]
pub async fn move_step(id: i64, up: bool) -> Result<(), ServerFnError> {
    let pool = ssr::db()?;
    let mut tx = pool.begin().await.map_err(ssr::err)?;

    let (process_id, position, parent_step_id, branch): (i64, i32, Option<i64>, Option<String>) =
        sqlx::query_as(
            "SELECT process_id, position, parent_step_id, branch FROM process_steps WHERE id = $1",
        )
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(ssr::err)?;

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
        .map_err(ssr::err)?;

    if let Some((neighbor_id, neighbor_position)) = neighbor {
        for (step_id, new_position) in [(id, neighbor_position), (neighbor_id, position)] {
            sqlx::query("UPDATE process_steps SET position = $2 WHERE id = $1")
                .bind(step_id)
                .bind(new_position)
                .execute(&mut *tx)
                .await
                .map_err(ssr::err)?;
        }
    }

    tx.commit().await.map_err(ssr::err)?;
    Ok(())
}


/// Schritte einer Ebene in ihrer Reihenfolge: Hauptablauf (`parent = None`) oder ein Ja-/Nein-Zweig.
pub fn child_steps(steps: &[ProcessStep], parent: Option<i64>, branch: Option<&str>) -> Vec<ProcessStep> {
    let mut children: Vec<ProcessStep> = steps
        .iter()
        .filter(|s| s.parent_step_id == parent && s.branch.as_deref() == branch)
        .cloned()
        .collect();
    children.sort_by_key(|s| (s.position, s.id));
    children
}

// ---------------------------------------------------------------------------
// Entitäten und ihre Spalten (1:N)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Entity {
    pub id: i64,
    pub name: String,
    /// Anzahl Spalten; nur in der Übersicht befüllt.
    pub column_count: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct EntityColumn {
    pub id: i64,
    pub entity_id: i64,
    pub name: String,
    pub data_type: String,
    pub meaning: String,
    /// Entität, auf die diese Spalte verweist (Fremdschlüssel).
    pub references_entity_id: Option<i64>,
}

#[server]
pub async fn list_entities() -> Result<Vec<Entity>, ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query_as(
        "SELECT e.id, e.name, COUNT(c.id) AS column_count
         FROM entities e LEFT JOIN entity_columns c ON c.entity_id = e.id
         GROUP BY e.id ORDER BY e.name",
    )
    .fetch_all(&pool)
    .await
    .map_err(ssr::err)
}

#[server]
pub async fn get_entity(id: i64) -> Result<Entity, ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query_as(
        "SELECT e.id, e.name, (SELECT COUNT(*) FROM entity_columns WHERE entity_id = e.id) AS column_count
         FROM entities e WHERE e.id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(ssr::err)?
    .ok_or_else(|| ServerFnError::new("Entität nicht gefunden"))
}

#[server]
pub async fn create_entity(name: String) -> Result<i64, ServerFnError> {
    ssr::required(&name, "Bezeichnung")?;
    let pool = ssr::db()?;
    sqlx::query_scalar("INSERT INTO entities (name) VALUES ($1) RETURNING id")
        .bind(name.trim())
        .fetch_one(&pool)
        .await
        .map_err(ssr::err)
}

#[server]
pub async fn rename_entity(id: i64, name: String) -> Result<(), ServerFnError> {
    ssr::required(&name, "Bezeichnung")?;
    let pool = ssr::db()?;
    sqlx::query("UPDATE entities SET name = $2 WHERE id = $1")
        .bind(id)
        .bind(name.trim())
        .execute(&pool)
        .await
        .map_err(ssr::err)?;
    Ok(())
}

#[server]
pub async fn delete_entity(id: i64) -> Result<(), ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query("DELETE FROM entities WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(ssr::err)?;
    Ok(())
}

#[server]
pub async fn list_entity_columns(entity_id: i64) -> Result<Vec<EntityColumn>, ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query_as(
        "SELECT id, entity_id, name, data_type, meaning, references_entity_id
         FROM entity_columns WHERE entity_id = $1 ORDER BY position, id",
    )
    .bind(entity_id)
    .fetch_all(&pool)
    .await
    .map_err(ssr::err)
}

/// Alle Spalten aller Entitäten, für das ER-Diagramm.
#[server]
pub async fn list_all_entity_columns() -> Result<Vec<EntityColumn>, ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query_as(
        "SELECT id, entity_id, name, data_type, meaning, references_entity_id
         FROM entity_columns ORDER BY entity_id, position, id",
    )
    .fetch_all(&pool)
    .await
    .map_err(ssr::err)
}

#[server]
pub async fn add_entity_column(
    entity_id: i64,
    name: String,
    data_type: String,
    meaning: String,
    references_entity_id: Option<i64>,
) -> Result<(), ServerFnError> {
    ssr::required(&name, "Bezeichnung")?;
    let pool = ssr::db()?;
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
    .map_err(ssr::err)?;
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
    ssr::required(&name, "Bezeichnung")?;
    let pool = ssr::db()?;
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
    .map_err(ssr::err)?;
    Ok(())
}

#[server]
pub async fn delete_entity_column(id: i64) -> Result<(), ServerFnError> {
    let pool = ssr::db()?;
    sqlx::query("DELETE FROM entity_columns WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(ssr::err)?;
    Ok(())
}
