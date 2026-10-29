CREATE TABLE admin_users (
 id TEXT PRIMARY KEY, username TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL,
 enabled BIGINT NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)), created_at BIGINT NOT NULL, updated_at BIGINT NOT NULL
);
CREATE TABLE admin_sessions (
 id TEXT PRIMARY KEY, user_id TEXT NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
 created_at BIGINT NOT NULL, expires_at BIGINT NOT NULL
);
CREATE INDEX idx_admin_sessions_user ON admin_sessions(user_id,created_at);
CREATE INDEX idx_admin_sessions_expiry ON admin_sessions(expires_at);
