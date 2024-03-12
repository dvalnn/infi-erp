CREATE DOMAIN item_status AS varchar(10)
CHECK (VALUE IN ('pending', 'in_stock', 'delivered', 'consumed'));

CREATE TABLE IF NOT EXISTS items (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

  piece_kind piece_kind NOT NULL REFERENCES pieces(code),
  order_id uuid REFERENCES orders(id),
  location char(2) REFERENCES warehouses(code),
  status item_status NOT NULL DEFAULT 'pending',
  acc_cost money NOT NULL DEFAULT 0,

  CHECK ((status = 'in_stock' AND location IS NOT NULL) OR (location IS NULL)),
  CHECK (( status = 'delivered' AND order_id IS NOT NULL) OR (status <> 'delivered'))
);
