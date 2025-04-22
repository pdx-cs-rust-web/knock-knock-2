-- https://www.shuttle.rs/blog/2023/10/04/sql-in-rust

CREATE TABLE IF NOT EXISTS jokes (
  id TEXT PRIMARY KEY,
  whos_there TEXT NOT NULL,
  answer_who TEXT NOT NULL
);
