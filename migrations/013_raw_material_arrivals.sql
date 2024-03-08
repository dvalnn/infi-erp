CREATE TABLE IF NOT EXISTS raw_material_arrivals (
  raw_material_id bigint PRIMARY KEY NOT NULL REFERENCES items(id),
  shippment_id bigint NOT NULL REFERENCES shippments(id),
  arrival_date int NOT NULL
);
