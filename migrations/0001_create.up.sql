-- https://www.shuttle.rs/blog/2023/10/04/sql-in-rust

CREATE TABLE IF NOT EXISTS jokes (
  whos_there VARCHAR(200) NOT NULL,
  answer_who VARCHAR(200) NOT NULL
);
