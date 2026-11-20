ALTER TABLE process_steps
    ADD COLUMN condition TEXT NOT NULL DEFAULT '',
    ADD COLUMN yes_next  TEXT NOT NULL DEFAULT '',
    ADD COLUMN no_next   TEXT NOT NULL DEFAULT '';

-- Antworten auf die Frage in `process_steps.condition`, z. B. "Summe > 1000.-" -> Ja.
CREATE TABLE condition_rules (
    id        BIGSERIAL PRIMARY KEY,
    step_id   BIGINT NOT NULL REFERENCES process_steps(id) ON DELETE CASCADE,
    position  INT NOT NULL,
    criterion TEXT NOT NULL,
    answer    TEXT NOT NULL CHECK (answer IN ('Ja', 'Nein'))
);

CREATE INDEX condition_rules_step_id_idx ON condition_rules (step_id, position);
