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
    ON DELETE CASCADE,

  UNIQUE(client_id, order_number)
);

CREATE FUNCTION check_work_piece_kind() RETURNS trigger AS
$$
  BEGIN
    IF NOT EXISTS (
      SELECT 1
      FROM pieces
      WHERE id = NEW.work_piece AND kind = 'final product'
    ) THEN
      RAISE EXCEPTION 'Work piece must be a final product.';
    END IF;
    RETURN NULL;
  END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER check_work_piece_kind_trigger
BEFORE INSERT ON orders
FOR EACH ROW EXECUTE PROCEDURE check_work_piece_kind();

