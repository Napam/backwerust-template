CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT);

INSERT INTO users (name)
VALUES ('I come from the migration file at db/migrations/0001_init.sql');
