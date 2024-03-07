CREATE TABLE IF NOT EXISTS shippments (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  supplier_id bigint NOT NULL REFERENCES suppliers(id),
  request_date int NOT NULL,
  quantity int NOT NULL,
  cost money NOT NULL
);
