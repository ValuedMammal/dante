CREATE TABLE latin (
    id serial PRIMARY KEY,
    en VARCHAR (32) UNIQUE NOT NULL,
    la VARCHAR (32) NOT NULL,
    defn VARCHAR (64) NOT NULL,
    fr VARCHAR (32) NOT NULL,
    es VARCHAR (32) NOT NULL,
    it VARCHAR (32) NOT NULL
);
-- Postgres
-- insert into latin (en, la, defn, fr, es, it) values ('absent', 'absens', '(adj) not present', 'absent', 'ausente', 'assente');
