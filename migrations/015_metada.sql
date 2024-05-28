CREATE TABLE IF NOT EXISTS epoch_table(
simulation_date INT
  NOT NULL
  DEFAULT 1
  CHECK(simulation_date >= 0)
  PRIMARY KEY
);

INSERT INTO epoch_table(simulation_date) VALUES(1);

CREATE FUNCTION run_daily_order_complete_check() RETURNS TRIGGER AS $$
BEGIN
  EXECUTE complete_all_producing_orders();
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Runs the check at the start of each simulation day
CREATE TRIGGER complete_producing_orders_after_update
AFTER UPDATE ON epoch_table
FOR EACH ROW
EXECUTE FUNCTION run_daily_order_complete_check();
