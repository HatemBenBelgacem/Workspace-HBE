-- Ja/Nein verweisen nicht mehr auf Folgeschritte; stattdessen hat jede Bedingung
-- eigene Schritte im Ja- und im Nein-Zweig.
ALTER TABLE process_steps
    DROP COLUMN yes_next,
    DROP COLUMN no_next,
    ADD COLUMN parent_step_id BIGINT REFERENCES process_steps(id) ON DELETE CASCADE,
    ADD COLUMN branch TEXT CHECK (branch IN ('Ja', 'Nein')),
    ADD CONSTRAINT process_steps_branch_parent CHECK ((parent_step_id IS NULL) = (branch IS NULL));

CREATE INDEX process_steps_parent_idx ON process_steps (parent_step_id, branch, position);

-- Die Antworttabellen werden durch die Zweig-Tabellen ersetzt.
DROP TABLE condition_rules;
