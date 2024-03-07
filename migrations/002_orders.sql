CREATE TABLE IF NOT EXISTS orders (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  client_id uuid NOT NULL REFERENCES clients(id),
  placement_day int NOT NULL,
  number int NOT NULL CHECK (due_date > 0),
  due_date int NOT NULL CHECK (due_date > 0),
  early_penalty money NOT NULL,
  late_penalty money NOT NULL
);


