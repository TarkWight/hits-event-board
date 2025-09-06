-- связь many-to-many: студенты ↔ события
CREATE TABLE IF NOT EXISTS event_registrations
(
    event_id      uuid        NOT NULL REFERENCES events (id) ON DELETE CASCADE,
    student_id    uuid        NOT NULL REFERENCES students (user_id) ON DELETE CASCADE,
    registered_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (event_id, student_id)
);

CREATE INDEX IF NOT EXISTS ix_event_registrations_student ON event_registrations (student_id);
CREATE INDEX IF NOT EXISTS ix_event_registrations_event ON event_registrations (event_id);