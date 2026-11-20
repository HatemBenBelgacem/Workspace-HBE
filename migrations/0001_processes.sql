CREATE TABLE processes (
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE process_steps (
    id         BIGSERIAL PRIMARY KEY,
    process_id BIGINT NOT NULL REFERENCES processes(id) ON DELETE CASCADE,
    position   INT NOT NULL,
    activity   TEXT NOT NULL,
    role       TEXT NOT NULL DEFAULT '',
    input      TEXT NOT NULL DEFAULT '',
    output     TEXT NOT NULL DEFAULT '',
    system     TEXT NOT NULL DEFAULT ''
);

CREATE INDEX process_steps_process_id_idx ON process_steps (process_id, position);
