CREATE TABLE IF NOT EXISTS suppliers (
  supplier_id varchar NOT NULL primary key,
  name varchar NOT NULL,
  piece varchar NOT NULL,
  min_order int NOT NULL,
  price_per_piece money NOT NULL,
  delivery_time interval NOT NULL
);

INSERT INTO suppliers (
  supplier_id, name, piece, min_order, price_per_piece, delivery_time
) VALUES ('A_P1', 'Supplier A', 'P1', 16, '$30', '4 days');

INSERT INTO suppliers (
  supplier_id, name, piece, min_order, price_per_piece, delivery_time
) VALUES ('A_P2', 'Supplier A', 'P2', 16, '$10', '4 days');

INSERT INTO suppliers (
  supplier_id, name, piece, min_order, price_per_piece, delivery_time
) VALUES ('B_P1', 'Supplier B', 'P1', 8, '$45', '2 days');

INSERT INTO suppliers (
  supplier_id, name, piece, min_order, price_per_piece, delivery_time
) VALUES ('B_P2', 'Supplier B', 'P2', 8, '$15', '2 days');

INSERT INTO suppliers (
  supplier_id, name, piece, min_order, price_per_piece, delivery_time
) VALUES ('C_P1', 'Supplier C', 'P1', 4, '$55', '1 days');

INSERT INTO suppliers (
  supplier_id, name, piece, min_order, price_per_piece, delivery_time
) VALUES ('C_P2', 'Supplier C', 'P2', 4, '$18', '1 days');
