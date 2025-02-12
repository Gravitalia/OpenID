-- Public keys table logic.

CREATE TABLE IF NOT EXISTS keys (
  id            SERIAL  PRIMARY KEY,
  user_id       TEXT    REFERENCES users(vanity) ON DELETE CASCADE,
  key           TEXT    NOT NULL,
  created_at    DATE    NOT NULL DEFAULT NOW()
);
