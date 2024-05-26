CREATE TYPE transformation_status AS ENUM ('pending', 'completed');

CREATE TABLE IF NOT EXISTS transformations(
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  material_id uuid NOT NULL REFERENCES items(id),
  product_id uuid NOT NULL REFERENCES items(id),
  recipe_id bigint NOT NULL REFERENCES recipes(id),
  status transformation_status NOT NULL DEFAULT 'pending',

  -- Metadata for vistualization purposes
  date int,
  time_taken int,
  machine char(2) REFERENCES machines(code),
  line char(2) REFERENCES production_lines(code)
);
