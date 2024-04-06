CREATE TABLE IF NOT EXISTS raw_material_shippments(
  raw_material_id uuid PRIMARY KEY NOT NULL REFERENCES items(id),
  shippment_id bigint NOT NULL REFERENCES shippments(id)
);
