CREATE TABLE IF NOT EXISTS client_orders (
  order_number BIGINT NOT NULL PRIMARY KEY,
  client_name_id VARCHAR NOT NULL,
  work_piece VARCHAR NOT NULL,
  quantity INT NOT NULL,
  due_date INT NOT NULL,
  late_pen MONEY NOT NULL,
  early_pen MONEY NOT NULL
);
