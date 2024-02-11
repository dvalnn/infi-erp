CREATE TABLE IF NOT EXISTS mps ( -- Master Production Schedule
  due_date int NOT NULL primary key,
  order_number bigint NOT NULL references client_orders(order_number),
  supplier varchar NOT NULL references suppliers(supplier_id)
);
