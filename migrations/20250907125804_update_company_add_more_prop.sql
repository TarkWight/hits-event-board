-- Add migration script here
DO $$
    BEGIN
        CREATE TYPE company_status AS ENUM ('active', 'archived');
    EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

ALTER TABLE companies
    ADD COLUMN IF NOT EXISTS description text NULL,
    ADD COLUMN IF NOT EXISTS website     text NULL,
    ADD COLUMN IF NOT EXISTS status      company_status NOT NULL DEFAULT 'active';

CREATE INDEX IF NOT EXISTS ix_companies_name_ci ON companies (lower(name));
