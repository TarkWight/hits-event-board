-- события
CREATE TABLE IF NOT EXISTS events
(
    id              uuid PRIMARY KEY,
    company_id      uuid        NOT NULL REFERENCES companies (id) ON DELETE RESTRICT,
    manager_id      uuid        NOT NULL REFERENCES managers (user_id) ON DELETE RESTRICT,
    title           text        NOT NULL CHECK (length(trim(title)) > 0),
    description     text        NULL,
    location        text        NULL,
    starts_at       timestamptz NOT NULL,
    ends_at         timestamptz NULL,
    signup_deadline timestamptz NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),

    -- инварианты домена
    CONSTRAINT chk_event_ends_after_start
        CHECK (ends_at IS NULL OR ends_at >= starts_at),
    CONSTRAINT chk_event_deadline_before_start
        CHECK (signup_deadline IS NULL OR signup_deadline <= starts_at)
);

DROP TRIGGER IF EXISTS set_events_updated_at ON events;
CREATE TRIGGER set_events_updated_at
    BEFORE UPDATE
    ON events
    FOR EACH ROW
EXECUTE FUNCTION trg_set_updated_at();

-- индексы для типичных запросов
CREATE INDEX IF NOT EXISTS ix_events_company ON events (company_id);
CREATE INDEX IF NOT EXISTS ix_events_manager ON events (manager_id);
CREATE INDEX IF NOT EXISTS ix_events_starts_at ON events (starts_at);
CREATE INDEX IF NOT EXISTS ix_events_deadline ON events (signup_deadline);