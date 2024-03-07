CREATE TABLE IF NOT EXISTS orders (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,

  client_id uuid NOT NULL REFERENCES clients(id),
  number int NOT NULL UNIQUE CHECK (number> 0),
  piece char(2) NOT NULL REFERENCES pieces(code),
  quantity int NOT NULL CHECK (quantity > 0),
  due_date int NOT NULL CHECK (due_date > 0),
  late_penalty money NOT NULL,
  early_penalty money NOT NULL,

  placement_day int NOT NULL
);


