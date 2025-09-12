-- Add статусы регистраций + GCal
DO $$
    BEGIN
        CREATE TYPE registration_status AS ENUM ('registered', 'canceled', 'attended', 'no_show');
    EXCEPTION WHEN duplicate_object THEN NULL;
    END $$;

-- Переименовывать не обязательно, но часто удобнее коротко
ALTER TABLE event_registrations RENAME TO registrations;

-- Добавляем колонки
ALTER TABLE registrations
    ADD COLUMN IF NOT EXISTS status registration_status NOT NULL DEFAULT 'registered',
    ADD COLUMN IF NOT EXISTS canceled_at timestamptz NULL,
    ADD COLUMN IF NOT EXISTS gcal_event_id text NULL;

-- Индексы для подсчёта занятых мест
CREATE INDEX IF NOT EXISTS ix_registrations_event_registered
    ON registrations (event_id)
    WHERE status = 'registered';
