CREATE TABLE logs (
    time INTEGER PRIMARY KEY NOT NULL,
    server_name TEXT NOT NULL,
    rps INTEGER NOT NULL
);

CREATE TABLE servers (
    server_id TEXT PRIMARY KEY NOT NULL,
    category TEXT NOT NULL,
    server_name TEXT NOT NULL,
    url TEXT NOT NULL
);