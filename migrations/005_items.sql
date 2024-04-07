CREATE TYPE item_status AS enum(
  'pending',
  'in_transit',
  'in_stock',
  'delivered',
  'consumed');

CREATE TABLE IF NOT EXISTS items (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

  piece_kind piece_kind NOT NULL REFERENCES pieces(code),
  order_id uuid REFERENCES orders(id),
  warehouse char(2) REFERENCES warehouses(code),
  production_line char(2) REFERENCES production_lines(code),
  status item_status NOT NULL DEFAULT 'pending',
  acc_cost money NOT NULL DEFAULT 0,

  CHECK (( status = 'delivered' AND order_id IS NOT NULL) OR (status <> 'delivered'))
);
