-- Eine Spalte kann auf eine andere Entität verweisen (Fremdschlüssel). Daraus ergeben sich die
-- Verbindungen im ER-Diagramm: viele Zeilen der Entität mit der Spalte gehören zu einer der anderen.
ALTER TABLE entity_columns
    ADD COLUMN references_entity_id BIGINT REFERENCES entities(id) ON DELETE SET NULL;
