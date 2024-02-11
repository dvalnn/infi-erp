CREATE TABLE IF NOT EXISTS client_orders (
  order_number bigint NOT NULL primary key,
  client_name_id varchar NOT NULL,
  work_piece varchar NOT NULL,
  quantity int NOT NULL,
  due_date int NOT NULL,
  late_pen money NOT NULL,
  early_pen money NOT NULL
);
