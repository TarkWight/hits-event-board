-- доменные enum'ы
DO
$$
    BEGIN
        CREATE TYPE user_role AS ENUM ('student', 'manager', 'dean');
    EXCEPTION
        WHEN duplicate_object THEN NULL;
    END
$$;

DO
$$
    BEGIN
        CREATE TYPE manager_status AS ENUM ('pending', 'confirmed', 'rejected');
    EXCEPTION
        WHEN duplicate_object THEN NULL;
    END
$$;