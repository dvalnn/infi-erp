CREATE VIEW order_details AS
SELECT
  c.name AS client_name,
  o.order_number,
  p.name AS piece_name,
  o.quantity,
  o.due_date,
  o.early_penalty,
  o.late_penalty
FROM orders o
INNER JOIN clients c ON c.id = o.client_id
INNER JOIN pieces p ON p.id = o.work_piece;
