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
  location char(2),
  status item_status NOT NULL DEFAULT 'pending',
  acc_cost money NOT NULL DEFAULT 0,

  CHECK (( status = 'delivered' AND order_id IS NOT NULL) OR (status <> 'delivered'))
);

-- CREATE FUNCTION check_location()
-- RETURNS TRIGGER AS $$
--   BEGIN
--     IF NEW.location NOT IN (SELECT code FROM production_lines UNION SELECT code FROM warehouses)
--     THEN
--       RAISE EXCEPTION 'Location % does not exist', NEW.location;
--     END IF;
--     RETURN NEW;
--   END
-- $$ LANGUAGE plpgsql;
--
-- CREATE TRIGGER check_location
-- BEFORE INSERT OR UPDATE ON items
-- FOR EACH ROW
-- EXECUTE FUNCTION check_location();


CREATE FUNCTION upsert_item()
RETURNS TRIGGER AS $$
  DECLARE free_stock RECORD;
  BEGIN
    IF NEW.order_id IS NULL THEN
      RETURN NEW; -- Insert as usual, no need to allocate stock
    END IF;

    IF NEW.piece_kind NOT IN ( SELECT code FROM pieces WHERE category = 'raw') THEN
      RETURN NEW; -- Insert as usual, we are only looking for raw materials
    END IF;

    SELECT * INTO free_stock FROM items As i
    WHERE i.piece_kind = NEW.piece_kind
      AND i.status = 'in_stock'
      AND i.order_id IS NULL
    LIMIT 1;

    IF NOT FOUND THEN
      RETURN NEW; -- Insert as usual
    END IF;

    UPDATE items
    SET id = new.id, -- HACK: this is a workaround to not break the application code
    order_id = new.order_id
    WHERE id = free_stock.id;

    UPDATE raw_material_shipments
    SET raw_material_id = new.id -- HACK: since the id was updated we need to update the reference
    WHERE raw_material_id = free_stock.id;

    RAISE NOTICE 'Item % alocated from existing stock to order %', free_stock.id, new.order_id;

    RETURN NULL; -- Do not insert, we updated an existing item
  END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER upsert_item
BEFORE INSERT ON items
FOR EACH ROW
EXECUTE FUNCTION upsert_item();


CREATE FUNCTION check_if_order_is_completed()
RETURNS TRIGGER AS $$
  DECLARE n_ready int;
  DECLARE eq_order RECORD;
  BEGIN

    IF NEW.status <> 'in_stock' THEN
      RETURN NEW;
    END IF;

    IF NEW.order_id IS NULL THEN
      RETURN NEW;
    END IF;

    SELECT * INTO eq_order FROM orders WHERE orders.id = NEW.order_id;
    IF NEW.piece_kind <> eq_order.piece THEN
      RETURN NEW;
    END IF;

    IF eq_order.status <> 'producing' THEN
      RAISE EXCEPTION 'final piece is in stock but order is not producing';
    END IF;

    SELECT COUNT(items.*) INTO n_ready FROM items
    WHERE items.order_id = eq_order.id
      AND items.piece_kind = eq_order.piece
      AND items.status = 'in_stock';

    IF n_ready < eq_order.quantity THEN
      RETURN NEW;
    END IF;

    IF n_ready > eq_order.quantity THEN
      RAISE EXCEPTION 'more items exist than requested by order %', eq_order.id;
    END IF;

    UPDATE orders
    SET status = 'completed'
    WHERE id = eq_order.id;

    RAISE NOTICE 'Order % is completed', eq_order.id;

    RETURN NEW;
  END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER check_if_order_is_completed
AFTER UPDATE OF status ON items
FOR EACH ROW
EXECUTE FUNCTION check_if_order_is_completed();


CREATE FUNCTION deliver_items()
RETURNS TRIGGER AS $$
DECLARE eq_order RECORD;
  BEGIN
    IF NEW.status <> 'delivered' THEN
      RETURN NEW;
    END IF;

    SELECT * INTO eq_order FROM orders WHERE orders.id = NEW.id;
    IF eq_order IS NULL THEN
      RAISE EXCEPTION 'Order % does not exist', NEW.id;
    END IF;

    IF eq_order.status = 'canceled' THEN
      RAISE EXCEPTION 'Order % is canceled', NEW.id;
    END IF;

    IF eq_order.status = 'delivered' THEN
      RAISE EXCEPTION 'Order % was already delivered', NEW.id;
    END IF;

    IF eq_order.status <> 'completed' THEN
      RAISE EXCEPTION 'Order % is not yet completed', NEW.id;
    END IF;

    UPDATE items
    SET status = 'delivered', location = NUll
    WHERE order_id = NEW.id
      AND piece_kind = NEW.piece
      AND status = 'in_stock';
    RAISE NOTICE 'Order % is delivered', NEW.id;

    RETURN NEW;
  END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER deliver_items
BEFORE UPDATE OF status ON orders
FOR EACH ROW
EXECUTE FUNCTION deliver_items();


CREATE OR REPLACE FUNCTION is_order_completed(order_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
  n_ready int;
  order_data RECORD;
BEGIN
  -- Fetch order details
  SELECT * INTO order_data
  FROM orders
  WHERE orders.id = order_id;

  -- Check for invalid order ID or non-producing order status
  IF NOT FOUND OR order_data.status <> 'producing' THEN
    RETURN FALSE;
  END IF;

  -- Count 'in_stock' items for the order
  SELECT COUNT(*) INTO n_ready
  FROM items
  WHERE items.order_id = order_data.id
  AND items.piece_kind = order_data.piece
  AND items.status = 'in_stock';

  -- Check if item count matches order quantity
  RETURN n_ready = order_data.quantity;
END;
$$ LANGUAGE plpgsql;


CREATE OR REPLACE FUNCTION complete_all_producing_orders()
RETURNS VOID AS $$
DECLARE
  order_record RECORD;
  order_cursor CURSOR FOR SELECT id FROM orders WHERE status = 'producing';
BEGIN
  -- Open cursor
  OPEN order_cursor;

  -- Loop through orders
  LOOP
    FETCH order_cursor INTO order_record;
    EXIT WHEN NOT FOUND;

    -- Check completion using is_order_completed
    IF is_order_completed(order_record.id) THEN
      UPDATE orders SET status = 'completed' WHERE id = order_record.id;
      RAISE NOTICE 'Order % is completed', order_record.id;
    END IF;
  END LOOP;

  -- Close cursor
  CLOSE order_cursor;

  RETURN;
END;
$$ LANGUAGE plpgsql;

