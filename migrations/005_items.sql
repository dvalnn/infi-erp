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

CREATE FUNCTION check_if_order_is_completed()
RETURNS TRIGGER AS $$
  DECLARE n_ready int;
  DECLARE eq_order RECORD;
  BEGIN
    SELECT * INTO eq_order FROM orders WHERE orders.id = NEW.order_id;

    IF NEW.piece_kind = eq_order.piece THEN
      SELECT COUNT(items.*) INTO n_ready FROM items
      WHERE items.order_id = eq_order.id
        AND items.piece_kind = eq_order.piece
        AND items.status = 'in_stock';

      IF n_ready = eq_order.quantity THEN
        UPDATE orders
        SET status = 'completed'
        WHERE id = eq_order.id;

        RAISE NOTICE 'Order % is completed', eq_order.id;
      END IF;
    END IF;

    RETURN NEW;
  END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER check_if_order_is_completed
AFTER UPDATE OF status ON items
FOR EACH ROW
EXECUTE FUNCTION check_if_order_is_completed();

