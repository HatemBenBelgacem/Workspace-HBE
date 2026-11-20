//! Gemeinsame Hilfen für die Server-Funktionen aller Bereiche (nur auf dem Server).

use leptos::prelude::*;
use sqlx::PgPool;

pub fn pool() -> Result<PgPool, ServerFnError> {
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
