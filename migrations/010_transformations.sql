CREATE TABLE IF NOT EXISTS transformations(
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  material_id uuid NOT NULL REFERENCES items(id),
  product_id uuid NOT NULL REFERENCES items(id),
  date int
);
