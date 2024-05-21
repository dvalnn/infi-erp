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


CREATE FUNCTION upsert_item()
RETURNS TRIGGER AS $$
  DECLARE free_stock RECORD;
  BEGIN
    IF NEW.order_id IS NULL THEN
      RETURN NEW; -- Insert as usual, no need to allocate stock
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


CREATE FUNCTION deliver_items()
RETURNS TRIGGER AS $$
  BEGIN

    IF NEW.status = 'delivered' THEN
      UPDATE items
      SET status = 'delivered'
      WHERE order_id = NEW.id
        AND status = 'in_stock'
        AND piece_kind = NEW.piece;
      RAISE NOTICE 'Order % is delivered', NEW.id;
    END IF;

    RETURN NEW;
  END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER deliver_items
AFTER UPDATE OF status ON orders
FOR EACH ROW
EXECUTE FUNCTION deliver_items();
