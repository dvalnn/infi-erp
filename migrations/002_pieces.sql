CREATE TYPE piece_category AS ENUM ('raw', 'inter', 'final');
CREATE TYPE piece_kind AS ENUM ('P1', 'P2', 'P3', 'P4', 'P5', 'P6', 'P7', 'P8', 'P9');

CREATE TABLE IF NOT EXISTS pieces (
  code piece_kind PRIMARY KEY,
  category piece_category NOT NULL
);

INSERT INTO pieces (code, category)
VALUES ('P1', 'raw'),
       ('P2', 'raw'),
       ('P3', 'inter'),
       ('P4', 'inter'),
       ('P5', 'final'),
       ('P6', 'final'),
       ('P7', 'final'),
       ('P8', 'inter'),
       ('P9', 'final');

