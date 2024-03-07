CREATE DOMAIN item_status AS varchar(10)
CHECK (VALUE IN ('pending', 'in_stock', 'delivered'));

CREATE TABLE IF NOT EXISTS items (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  piece_kind char(2) NOT NULL REFERENCES pieces(code),
  status item_status NOT NULL DEFAULT 'pending',
  location char(2) REFERENCES warehouses(code),
  acc_cost money,

  CHECK ((status = 'in_stock' AND location IS NOT NULL) OR (location IS NULL)),
  CHECK ((status = 'pending' AND acc_cost IS NULL) OR (acc_cost IS NOT NULL))
);
