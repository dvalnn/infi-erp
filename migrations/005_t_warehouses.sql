CREATE TABLE IF NOT EXISTS warehouses(
  name VARCHAR NOT NULL,
  piece BIGINT NOT NULL,
  arrival_date BIGINT,
  departure_date BIGINT,

  PRIMARY KEY(piece, arrival_date),
  FOREIGN KEY(piece)
    REFERENCES pieces(id)
    ON DELETE CASCADE
)

