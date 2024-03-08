CREATE TYPE order_status AS ENUM ('pending', 'scheduled', 'delivered', 'canceled');

CREATE TABLE IF NOT EXISTS orders (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,

  client_id uuid NOT NULL REFERENCES clients(id),
  number int NOT NULL UNIQUE CHECK (number> 0),
  piece piece_kind NOT NULL REFERENCES pieces(code),
  quantity int NOT NULL CHECK (quantity > 0),
  due_date int NOT NULL CHECK (due_date > 0),
  late_penalty money NOT NULL,
  early_penalty money NOT NULL,

  status order_status NOT NULL DEFAULT 'pending',
  placement_day int NOT NULL,
  delivery_day int,

  CHECK (
    (status IN ('pending', 'canceled') AND delivery_day IS NULL) OR
    (status IN ('scheduled', 'delivered') AND delivery_day IS NOT NULL)
  )
);
