-- одноразовые коды для линка
CREATE TABLE IF NOT EXISTS telegram_link_codes (
    code        TEXT PRIMARY KEY,
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
    );

CREATE INDEX IF NOT EXISTS idx_tlc_user_id     ON telegram_link_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_tlc_expires_at  ON telegram_link_codes(expires_at);

-- ограничения для связей
CREATE UNIQUE INDEX IF NOT EXISTS uq_telegram_links_user ON telegram_links(user_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_telegram_links_tg   ON telegram_links(telegram_user_id);