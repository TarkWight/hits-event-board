-- Таблица refresh-токена Google (на пользователя)
CREATE TABLE IF NOT EXISTS google_accounts (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    refresh_token text NOT NULL,
    scope   text,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

-- Связка с Telegram user_id
CREATE TABLE IF NOT EXISTS telegram_links (
    user_id uuid    PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    telegram_user_id bigint UNIQUE NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);
