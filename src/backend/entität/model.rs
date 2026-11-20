use serde::{Deserialize, Serialize};

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
