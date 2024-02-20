
CREATE TABLE IF NOT EXISTS orders ( 
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  client_id BIGINT NOT NULL,
  order_number BIGINT NOT NULL,
  quantity BIGINT NOT NULL,
  due_date BIGINT NOT NULL,
  early_pen MONEY NOT NULL,
  late_pen MONEY NOT NULL,

  UNIQUE (client_id, order_number),

  FOREIGN KEY(client_id)
    REFERENCES clients(id)
    ON DELETE CASCADE
)

