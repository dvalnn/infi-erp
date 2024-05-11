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
  quantity int NOT NULL CHECK (quantity > 0),
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

CREATE FUNCTION deliver_items()
RETURNS TRIGGER AS $$
  BEGIN

    IF NEW.status = 'delivered' THEN
      UPDATE orders
      SET status = 'delivered'
      WHERE id = NEW.order_id
        AND status = 'in_stock'
        AND piece_kind = NEW.piece;
      RAISE NOTICE 'Order % is delivered', NEW.order_id;
    END IF;

    RETURN NEW;
  END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER deliver_items
AFTER UPDATE OF status ON orders
FOR EACH ROW
EXECUTE FUNCTION deliver_items();


CREATE VIEW order_final_costs AS (
  WITH penalties AS (
    SELECT
      orders.id,
      CASE
        WHEN orders.delivery_day > orders.due_date
          THEN orders.late_penalty * (orders.delivery_day - orders.due_date)
        ELSE orders.early_penalty * (orders.due_date - orders.delivery_day)
      END AS penalty
    FROM orders WHERE orders.status = 'delivered'
  )


  SELECT
    orders.id,
    SUM(items.acc_cost) + penalties.penalty AS final_cost
  FROM orders
  JOIN items ON orders.id = items.order_id
  JOIN penalties ON orders.id = penalties.id
  WHERE orders.status = 'delivered'
  GROUP BY orders.id
);
