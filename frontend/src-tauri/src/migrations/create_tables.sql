CREATE TABLE IF NOT EXISTS uploads (
    hash TEXT PRIMARY KEY,
    filename TEXT,
    size TEXT,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS tokens (
    token TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    ip TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS auth_codes (
    code TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    ip TEXT NOT NULL
);