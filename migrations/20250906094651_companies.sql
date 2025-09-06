-- companies
CREATE TABLE IF NOT EXISTS companies
(
    id         uuid PRIMARY KEY,
    name       text        NOT NULL CHECK (length(trim(name)) > 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

DROP TRIGGER IF EXISTS set_companies_updated_at ON companies;
CREATE TRIGGER set_companies_updated_at
    BEFORE UPDATE ON companies
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();

-- Добавляем FK только если его ещё нет
DO $$
    BEGIN
        IF NOT EXISTS (
            SELECT 1
            FROM pg_constraint c
            WHERE c.conname = 'fk_managers_company'
        ) THEN
            ALTER TABLE managers
                ADD CONSTRAINT fk_managers_company
                    FOREIGN KEY (company_id) REFERENCES companies (id)
                        ON DELETE RESTRICT;
        END IF;
    END $$;

-- Индексы (эти можно оставлять с IF NOT EXISTS)
CREATE INDEX IF NOT EXISTS ix_managers_company_id ON managers (company_id);
CREATE INDEX IF NOT EXISTS ix_managers_status     ON managers (status);
