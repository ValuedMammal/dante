CREATE TABLE latin (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    en TEXT NOT NULL UNIQUE,
    la TEXT NOT NULL,
    defn TEXT,
    fr TEXT,
    es TEXT,
    it TEXT
);
-- insert into latin (en, la, defn, fr, es, it) values ("absent", "absens", "(adj) not present", "absent", "ausente", "assente");