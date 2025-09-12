-- базовая таблица пользователей
CREATE TABLE IF NOT EXISTS users
(
    id                       uuid PRIMARY KEY,
    name                     text        NOT NULL CHECK (length(trim(name)) > 0),
    email                    citext      NOT NULL,
    password_hash            text        NOT NULL,
    refresh_token_hash       text        NULL,
    refresh_token_expiration timestamptz NULL,
    role                     user_role   NOT NULL,
    created_at               timestamptz NOT NULL DEFAULT now(),
    updated_at               timestamptz NOT NULL DEFAULT now()
);

-- уникальность email (без учёта регистра)
CREATE UNIQUE INDEX IF NOT EXISTS uq_users_email ON users (email);

-- триггеры обновления updated_at (опционально, но удобно)
CREATE OR REPLACE FUNCTION trg_set_updated_at() RETURNS trigger AS
$$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_users_updated_at ON users;
CREATE TRIGGER set_users_updated_at
    BEFORE UPDATE
    ON users
    FOR EACH ROW
EXECUTE FUNCTION trg_set_updated_at();