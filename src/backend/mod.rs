pub mod bpmn;
#[cfg(feature = "ssr")]
pub mod db;
// Rust erlaubt keine Umlaute in Modulnamen, der Ordner heisst wie in CLAUDE.md vorgegeben.
#[path = "entität/mod.rs"]
pub mod entitaet;
pub mod prozess;
