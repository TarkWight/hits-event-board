DO $$ BEGIN
    CREATE TYPE student_status AS ENUM ('created','linked','confirmed','rejected');
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

ALTER TABLE students
    ADD COLUMN IF NOT EXISTS status student_status NOT NULL DEFAULT 'created';

CREATE INDEX IF NOT EXISTS idx_students_status ON students(status);