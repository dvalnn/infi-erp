CREATE OR REPLACE VIEW orders_count_by_due_date AS
    SELECT due_date, COUNT(*) AS count
    FROM client_orders
    GROUP BY due_date;
