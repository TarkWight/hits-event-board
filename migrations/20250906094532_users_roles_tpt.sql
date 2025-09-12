-- таблица менеджеров (TPT)
CREATE TABLE IF NOT EXISTS managers
(
    user_id    uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    status     manager_status NOT NULL,
    company_id uuid           NOT NULL
    -- FK на companies добавим после создания companies (см. 0005)
    );

-- таблица студентов (TPT) — сейчас без доп. полей, но оставляем под расширение
CREATE TABLE IF NOT EXISTS students
(
    user_id uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE
    );

-- деаны инвариантно выражаются самой ролью в users.role = 'dean'