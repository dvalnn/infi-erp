CREATE TABLE IF NOT EXISTS orders(
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

  work_piece BIGINT NOT NULL,
  client_id BIGINT NOT NULL,
  order_number INT NOT NULL,
  quantity INT NOT NULL,
  due_date INT NOT NULL,
  late_penalty MONEY,
  early_penalty MONEY,

  FOREIGN KEY(work_piece) REFERENCES pieces(id),
  FOREIGN KEY(client_id)  REFERENCES clients(id)
);
