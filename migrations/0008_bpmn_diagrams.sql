-- Frei modellierte BPMN-2.0-Diagramme aus dem grafischen Editor.
-- Das Diagramm wird vollständig als BPMN-2.0-XML gespeichert (Austauschformat, z. B. für Camunda oder Signavio).
CREATE TABLE bpmn_diagrams (
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    xml         TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
