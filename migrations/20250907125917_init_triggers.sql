-- Гарантируем, что managers.user_id => users.role = 'manager'
CREATE OR REPLACE FUNCTION trg_managers_role_guard() RETURNS trigger AS $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM users u WHERE u.id = NEW.user_id AND u.role = 'manager') THEN
        RAISE EXCEPTION 'user % is not a manager', NEW.user_id;
    END IF;
    RETURN NEW;
END; $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS managers_role_guard ON managers;
CREATE TRIGGER managers_role_guard
    BEFORE INSERT OR UPDATE ON managers
    FOR EACH ROW EXECUTE FUNCTION trg_managers_role_guard();

-- Запретить создавать/менять событие с менеджером, который не confirmed
CREATE OR REPLACE FUNCTION trg_events_manager_must_be_confirmed() RETURNS trigger AS $$
DECLARE st manager_status;
BEGIN
    SELECT m.status INTO st FROM managers m WHERE m.user_id = NEW.manager_id;
    IF st IS NULL THEN
        RAISE EXCEPTION 'manager % not found', NEW.manager_id;
    ELSIF st <> 'confirmed' THEN
        RAISE EXCEPTION 'manager % must be confirmed (got %)', NEW.manager_id, st;
    END IF;
    RETURN NEW;
END; $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS events_manager_guard ON events;
CREATE TRIGGER events_manager_guard
    BEFORE INSERT OR UPDATE ON events
    FOR EACH ROW EXECUTE FUNCTION trg_events_manager_must_be_confirmed();
