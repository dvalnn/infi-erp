CREATE TABLE IF NOT EXISTS warehouses (
  code char(2) PRIMARY KEY,
  capacity int NOT NULL CHECK (capacity > 0)
);

INSERT INTO warehouses (code, capacity) VALUES ('W1', 32), ('W2', 32);

CREATE TABLE IF NOT EXISTS production_lines (
  code char(2) PRIMARY KEY
);

INSERT INTO production_lines (code) VALUES ('L0'),
                                           ('L1'),
                                           ('L2'),
                                           ('L3'),
                                           ('L4'),
                                           ('L5'),
                                           ('L6');
