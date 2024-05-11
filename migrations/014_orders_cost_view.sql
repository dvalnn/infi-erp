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
  GROUP BY orders.id, penalties.penalty
);
