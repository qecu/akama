CREATE TABLE IF NOT EXISTS account (
    jid TEXT PRIMARY KEY,
    resource TEXT,
    password TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS contact (
    account_jid TEXT NOT NULL,
    contact_jid TEXT NOT NULL,
    UNIQUE(account_jid, contact_jid)
);
CREATE TABLE IF NOT EXISTS message ( 
    "from" TEXT NOT NULL,
    "to" TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL
);
CREATE TABLE IF NOT EXISTS deliverd (
    accound_jid TEXT NOT NULL,
    contact_jid TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL
);
CREATE TABLE IF NOT EXISTS read_marker (
    accound_jid TEXT NOT NULL,
    contact_jid TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL
);
