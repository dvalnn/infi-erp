CREATE TABLE IF NOT EXISTS raw_materials(
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  associated_piece BIGINT NOT NULL,
  material_type VARCHAR(2) NOT NULL,
  supplier VARCHAR NOT NULL,
  cost MONEY NOT NULL,
  date_of_arrival BIGINT NOT NULL,

  UNIQUE(associated_piece),

  FOREIGN KEY(associated_piece)
    REFERENCES pieces(id)
    ON DELETE CASCADE
)
