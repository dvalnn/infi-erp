CREATE TABLE IF NOT EXISTS warehouses (
  code char(2) PRIMARY KEY,
  capacity int NOT NULL CHECK (capacity > 0)
);

INSERT INTO warehouses (code, capacity) VALUES ('W1', 32), ('W2', 32);
