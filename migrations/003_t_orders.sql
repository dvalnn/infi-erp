CREATE TABLE IF NOT EXISTS orders(
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

  work_piece BIGINT NOT NULL,
  client_id BIGINT NOT NULL,
  order_number INT NOT NULL,
  quantity INT NOT NULL,
  due_date INT NOT NULL,
  late_penalty MONEY NOT NULL DEFAULT 0,
  early_penalty MONEY NOT NULL DEFAULT 0,

  FOREIGN KEY(work_piece) REFERENCES pieces(id)
    ON DELETE CASCADE,
  FOREIGN KEY(client_id)  REFERENCES clients(id)
    ON DELETE CASCADE

  UNIQUE(client_id, order_number)

  CONSTRAINT check_work_piece_kind CHECK (
    work_piece IN (
      SELECT id FROM pieces WHERE kind = 'final product'
    )
  )
);
