CREATE TABLE IF NOT EXISTS suppliers (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  name varchar(50) NOT NULL,
  raw_material_kind piece_kind NOT NULL REFERENCES pieces(code),
  min_order_quantity integer NOT NULL CHECK(min_order_quantity > 0),
  unit_price money NOT NULL CHECK(unit_price > 0::money),
  delivery_time integer NOT NULL CHECK(delivery_time > 0)
);

INSERT INTO suppliers (name, raw_material_kind, min_order_quantity, unit_price ,delivery_time)
VALUES ('Supplier A', 'P1', 16, 30, 4),
       ('Supplier A', 'P2', 16, 10, 4),
       ('Supplier B', 'P1',  8, 45, 2),
       ('Supplier B', 'P2',  8, 15, 2),
       ('Supplier C', 'P1',  4, 55, 1),
       ('Supplier C', 'P2',  4, 18, 1);

