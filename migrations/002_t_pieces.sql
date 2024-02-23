CREATE DOMAIN piece_names AS VARCHAR(2) CHECK (value IN(
  'P1',
  'P2',
  'P3',
  'P4',
  'P5',
  'P6',
  'P7',
  'P8',
  'P9'
));

CREATE DOMAIN piece_kinds AS VARCHAR CHECK (value IN(
  'raw material',
  'intermediate',
  'final product'
));

CREATE TABLE IF NOT EXISTS pieces (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

  name piece_names NOT NULL,
  kind piece_kinds NOT NULL,

  UNIQUE(name)
);

INSERT INTO pieces (name, kind) VALUES ('P1', 'raw material');
INSERT INTO pieces (name, kind) VALUES ('P2', 'raw material');
INSERT INTO pieces (name, kind) VALUES ('P3', 'intermediate');
INSERT INTO pieces (name, kind) VALUES ('P4', 'intermediate');
INSERT INTO pieces (name, kind) VALUES ('P5', 'final product');
INSERT INTO pieces (name, kind) VALUES ('P6', 'final product');
INSERT INTO pieces (name, kind) VALUES ('P7', 'final product');
INSERT INTO pieces (name, kind) VALUES ('P8', 'intermediate');
INSERT INTO pieces (name, kind) VALUES ('P9', 'final product');

