CREATE TYPE order_status AS ENUM (
  'pending',
  'scheduled',
  'producing',
  'completed',
  'delivered',
  'canceled');

CREATE TABLE IF NOT EXISTS orders (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  client_id uuid NOT NULL REFERENCES clients(id),
  number int NOT NULL CHECK (number> 0),
  piece piece_kind NOT NULL REFERENCES pieces(code),
  quantity int NOT NULL CHECK (quantity > 0 AND quantity <= 24),
  due_date int NOT NULL CHECK (due_date > 0),
  late_penalty money NOT NULL,
  early_penalty money NOT NULL,

  status order_status NOT NULL DEFAULT 'pending',
  placement_day int NOT NULL,
  delivery_day int,

  UNIQUE (client_id, number),

  CHECK (
      (status IN ('pending', 'canceled') AND delivery_day IS NULL)
    OR
      delivery_day IS NOT NULL
  )
);
