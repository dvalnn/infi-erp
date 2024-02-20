CREATE TABLE IF NOT EXISTS pieces(
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  associated_order BIGINT NOT NULL,
  final_type VARCHAR(2) NOT NULL,
  current_type VARCHAR(2) NOT NULL,
  acc_cost MONEY NOT NULL,
  event_log JSON,

  FOREIGN KEY(associated_order)
    REFERENCES orders(id)
    ON DELETE CASCADE
)
