CREATE DOMAIN piece_type AS varchar(5)
CHECK (VALUE IN ('raw', 'inter', 'final'));

CREATE TABLE IF NOT EXISTS pieces (
  code char(2) PRIMARY KEY,
  type piece_type NOT NULL
);

INSERT INTO pieces (code, type)
VALUES ('P1', 'raw'),
       ('P2', 'raw'),
       ('P3', 'inter'),
       ('P4', 'inter'),
       ('P5', 'final'),
       ('P6', 'final'),
       ('P7', 'final'),
       ('P8', 'inter'),
       ('P9', 'final');

