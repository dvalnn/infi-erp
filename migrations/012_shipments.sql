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
  BEGIN

    SELECT unit_price INTO item_price
    FROM suppliers
    JOIN shipments AS sh ON sh.supplier_id = suppliers.id
    WHERE sh.id = NEW.id;

    UPDATE items
    SET status = 'in_stock',
        warehouse = 'W1',
        acc_cost = item_price
    WHERE id IN
    (
        SELECT items.id
        FROM items
        JOIN raw_material_shipments AS rs
            ON rs.raw_material_id = items.id
        JOIN shipments AS s
            ON rs.shipment_id = s.id
        WHERE s.id = NEW.id
    );

    RAISE NOTICE 'Shipment % arrived', NEW.id;
    RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER shipment_arrived_trigger
AFTER UPDATE OF arrival_date ON shipments
FOR EACH ROW
EXECUTE FUNCTION shipment_arrived();
