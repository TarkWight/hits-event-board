-- Add migration script here
CREATE TABLE IF NOT EXISTS idempotency_keys (
    key text PRIMARY KEY,
    user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    method  text NOT NULL,
    path    text NOT NULL,
    request_hash    bytea,
    response_status integer,
    response_body   jsonb,
    locked_until    timestamptz,
    created_at  timestamptz NOT NULL DEFAULT now(),
    processed_at    timestamptz
);
