CREATE OR REPLACE FUNCTION get_raw_material(item_id uuid)
RETURNS uuid AS $$
DECLARE curr_id uuid;
        next_id uuid;
BEGIN
  SELECT item_id INTO curr_id;

  LOOP
    SELECT material_id INTO next_id
    FROM transformations
    WHERE product_id = curr_id;
    EXIT WHEN NOT FOUND;

    SELECT next_id INTO curr_id;
  END LOOP;

  RETURN curr_id;
END;
$$
LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION item_cost(item_id uuid) RETURNS money AS $$
DECLARE raw_material RECORD;
        material_cost MONEY;
        material_arrival_date INT;
        dispatch_date INT;
        accumulated_cost MONEY;
BEGIN
  SELECT acc_cost INTO accumulated_cost
  FROM items
  WHERE id = item_id;

  SELECT * INTO raw_material
  FROM items
  WHERE id = get_raw_material(item_id);

  SELECT delivery_day INTO dispatch_date
  FROM orders
  WHERE id = raw_material.order_Id;

  SELECT s.cost, s.arrival_date INTO material_cost, material_arrival_date
  FROM shipments AS s
  JOIN raw_material_shipments AS rs
      ON rs.shipment_id = s.id
  WHERE rs.raw_material_id = raw_material.id;

  RETURN accumulated_cost + material_cost +
        CAST(CAST(material_cost AS numeric)
            * (dispatch_date - material_arrival_date) * 0.01 AS MONEY);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE VIEW order_final_costs AS (
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
  SELECT o.id AS order_id,
         SUM(item_cost(i.id)) + penalties.penalty AS cost
  FROM orders AS o
  JOIN items AS i
      ON i.order_id = o.id
  JOIN penalties
      ON penalties.id = o.id
  WHERE i.piece_kind = o.piece
        AND o.status = 'delivered'
  GROUP BY o.id, penalties.penalty
);
