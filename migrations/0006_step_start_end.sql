-- Start- und Endpunkte eines Prozesses werden in der Tabelle festgelegt statt automatisch bestimmt.
-- Ein Prozess hat höchstens einen Startpunkt, aber beliebig viele Endpunkte (z. B. am Ende eines Zweigs).
ALTER TABLE process_steps
    ADD COLUMN is_start BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN is_end   BOOLEAN NOT NULL DEFAULT false;

CREATE UNIQUE INDEX process_steps_one_start_idx ON process_steps (process_id) WHERE is_start;
