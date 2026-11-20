CREATE TABLE entities (
    id         BIGSERIAL PRIMARY KEY,
    name       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Eine Entität hat beliebig viele Spalten (1:N).
CREATE TABLE entity_columns (
    id        BIGSERIAL PRIMARY KEY,
    entity_id BIGINT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    position  INT NOT NULL,
    name      TEXT NOT NULL,
    data_type TEXT NOT NULL DEFAULT '',
    meaning   TEXT NOT NULL DEFAULT ''
);

CREATE INDEX entity_columns_entity_id_idx ON entity_columns (entity_id, position);
