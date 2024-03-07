CREATE DOMAIN delivery_status AS varchar(10)
CHECK (VALUE IN ('pending', 'scheduled', 'delivered', 'cancelled'));

CREATE TABLE IF NOT EXISTS deliveries (
  order_id bigint PRIMARY KEY REFERENCES orders(id),
  status delivery_status NOT NULL DEFAULT 'pending',
  delivery_day int,
  -- delivery day is not null for scheduled and delivered deliveries
  -- delivery day is null for pending and cancelled deliveries
  CHECK (
    (status IN ('pending', 'cancelled') AND delivery_day IS NULL) OR
    (status IN ('scheduled', 'delivered') AND delivery_day IS NOT NULL)
  )
);
