-- публикация и вместимость в events
ALTER TABLE events
    ADD COLUMN IF NOT EXISTS capacity integer NULL CHECK (capacity IS NULL OR capacity >= 1),
    ADD COLUMN IF NOT EXISTS is_published boolean NOT NULL DEFAULT false;

-- Индексы под выборки
CREATE INDEX IF NOT EXISTS ix_events_published ON events (is_published);
CREATE INDEX IF NOT EXISTS ix_events_company_time ON events (company_id, starts_at DESC);
