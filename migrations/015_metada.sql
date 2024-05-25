CREATE TABLE IF NOT EXISTS epoch_table(
simulation_date INT
  NOT NULL
  DEFAULT 1
  CHECK(simulation_date >= 0)
  PRIMARY KEY
);
