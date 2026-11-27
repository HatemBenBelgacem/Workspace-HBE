-- BPMN-2.0-Elemente in der Schritt-Tabelle:
-- Art des Elements (Aufgabe, Teilprozess, Zwischenereignis), Aufgabentyp, Ereignistyp,
-- Wiederholung (Schleife, Mehrfachinstanz), Art der Verzweigung (exklusiv, parallel)
-- sowie Auslöser und Bezeichnung von Start- und Endereignis.
ALTER TABLE process_steps
    ADD COLUMN element           TEXT NOT NULL DEFAULT 'Aufgabe'
        CHECK (element IN ('Aufgabe', 'Teilprozess', 'Zwischenereignis')),
    ADD COLUMN task_type         TEXT NOT NULL DEFAULT '',
    ADD COLUMN event_type        TEXT NOT NULL DEFAULT '',
    ADD COLUMN loop_type         TEXT NOT NULL DEFAULT '',
    ADD COLUMN gateway           TEXT NOT NULL DEFAULT ''
        CHECK (gateway IN ('', 'Exklusiv', 'Parallel')),
    ADD COLUMN start_event       TEXT NOT NULL DEFAULT '',
    ADD COLUMN start_label       TEXT NOT NULL DEFAULT '',
    ADD COLUMN end_event         TEXT NOT NULL DEFAULT '',
    ADD COLUMN end_label         TEXT NOT NULL DEFAULT '',
    -- Ein Teilprozess kann einen anderen erfassten Prozess aufrufen (Aufruf-Aktivität).
    ADD COLUMN called_process_id BIGINT REFERENCES processes(id) ON DELETE SET NULL;

-- Bisherige Bedingungen sind exklusive Verzweigungen (Ja/Nein).
UPDATE process_steps s
SET gateway = 'Exklusiv'
WHERE s.condition <> ''
   OR EXISTS (SELECT 1 FROM process_steps c WHERE c.parent_step_id = s.id);
