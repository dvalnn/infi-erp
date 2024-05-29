CREATE TABLE IF NOT EXISTS shipments (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  supplier_id bigint NOT NULL REFERENCES suppliers(id),
  request_date int NOT NULL,
  arrival_date int,
  quantity int NOT NULL,
  cost money NOT NULL
);

CREATE FUNCTION shipment_arrived() RETURNS TRIGGER AS $$
DECLARE item_price money;
        item_ids uuid[];
        item_id uuid;
        new_item_id uuid;
        p_kind char(2);
        n_missing_items int;
  BEGIN
    SELECT unit_price, CAST(raw_material_kind AS char(2))
      INTO item_price, p_kind
    FROM suppliers
    JOIN shipments AS sh ON sh.supplier_id = suppliers.id
    WHERE sh.id = NEW.id;

    SELECT ARRAY_AGG(items.id) INTO item_ids
    FROM items
    JOIN raw_material_shipments AS rs
        ON rs.raw_material_id = items.id
    JOIN shipments AS s
        ON rs.shipment_id = s.id
    WHERE s.id = NEW.id;

    IF array_length(item_ids, 1) > 0 THEN
      FOREACH item_id IN ARRAY item_ids LOOP
        RAISE NOTICE 'Item % arrived', item_id;
        UPDATE items
        SET status = 'in_stock',
          location = 'W1'
        WHERE id = item_id;
      END LOOP;
    END IF;

    SELECT NEW.quantity - array_length(item_ids, 1) INTO n_missing_items;
    IF n_missing_items > 0 THEN
      RAISE NOTICE 'Missing items: %', n_missing_items;
      FOR i IN 1..n_missing_items
      LOOP
        INSERT INTO items (piece_kind, status, location)
        VALUES (CAST(p_kind AS piece_kind), 'in_stock', 'W1')
        RETURNING id INTO new_item_id;
      END LOOP;

      RAISE NOTICE '% free items added of type %', n_missing_items, p_kind;
    END IF;

    RETURN NEW;
  END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER shipment_arrived_trigger
AFTER UPDATE OF arrival_date ON shipments
FOR EACH ROW
EXECUTE FUNCTION shipment_arrived();
