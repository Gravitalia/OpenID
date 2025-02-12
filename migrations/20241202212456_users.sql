-- User table logic.

CREATE TABLE IF NOT EXISTS users (
  vanity        TEXT        PRIMARY KEY,
  username      TEXT        NOT NULL,
  email         TEXT        UNIQUE NOT NULL,
  password      TEXT        NOT NULL,
  avatar        TEXT,
  flags         INT         NOT NULL DEFAULT 0,
  suspended_at  DATE,
  deleted_at    DATE,
  created_at    DATE        NOT NULL DEFAULT NOW(), -- Millisecond precision seems... too precise
  updated_at    TIMESTAMPTZ
);

-- Create a trigger to automatically update "updated_at" row.
CREATE OR REPLACE FUNCTION set_updated_at()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION trigger_updated_at(tablename REGCLASS)
    RETURNS VOID AS
$$
BEGIN
    EXECUTE FORMAT('CREATE TRIGGER set_updated_at
        BEFORE UPDATE
        ON %s
        FOR EACH ROW
        WHEN (OLD IS DISTINCT FROM NEW)
    EXECUTE FUNCTION set_updated_at();', tablename);
END;
$$ LANGUAGE plpgsql;

SELECT trigger_updated_at('"users"');

-- Define a function to delete users based on grace period.
-- CREATE OR REPLACE FUNCTION clean_deleted_users()
--RETURNS VOID AS
--$$
--BEGIN
--    DELETE FROM users
--    WHERE (deleted_at IS NOT NULL AND deleted_at + INTERVAL '30 days' < NOW())
--       OR (suspended_at IS NOT NULL AND suspended_at + INTERVAL '15 days' < NOW());
--END;
--$$ LANGUAGE plpgsql;

--SELECT cron.schedule('daily_cleanup', '0 0 * * *', 'SELECT clean_deleted_users();');
