CREATE VIEW orders_count_by_duedate AS
    SELECT due_date, COUNT(*) AS count
    FROM client_orders
    GROUP BY due_date;
