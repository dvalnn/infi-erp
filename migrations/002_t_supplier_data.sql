CREATE TABLE IF NOT EXISTS suppliers (
  name             varchar  ,
  piece            varchar  ,
  min_order        int      NOT NULL,
  price_per_piece  money    NOT NULL,
  delivery_time    interval NOT NULL,

  PRIMARY KEY (name, piece)
);

INSERT INTO suppliers (
  name, piece, min_order, price_per_piece, delivery_time
) VALUES ('Supplier A', 'P1', 16, '$30', '4 days');

INSERT INTO suppliers (
  name, piece, min_order, price_per_piece, delivery_time
) VALUES ('Supplier A', 'P2', 16, '$10', '4 days');

INSERT INTO suppliers (
  name, piece, min_order, price_per_piece, delivery_time
) VALUES ('Supplier B', 'P1', 8, '$45', '2 days');

INSERT INTO suppliers (
  name, piece, min_order, price_per_piece, delivery_time
) VALUES ('Supplier B', 'P2', 8, '$15', '2 days');

INSERT INTO suppliers (
  name, piece, min_order, price_per_piece, delivery_time
) VALUES ('Supplier C', 'P1', 4, '$55', '1 days');

INSERT INTO suppliers (
  name, piece, min_order, price_per_piece, delivery_time
) VALUES ('Supplier C', 'P2', 4, '$18', '1 days');
