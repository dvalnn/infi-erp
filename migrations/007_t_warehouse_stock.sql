-- Per day stock projections for each warehouse.
-- Used to calculate the MPS
-- Dependent on the the production targets

CREATE TABLE IF NOT EXISTS w1_daily_stock(
  day         int NOT NULL primary key references mps(day),

  w1_p1_stock int NOT NULL,
  w1_p2_stock int NOT NULL,
  w1_p3_stock int NOT NULL,
  w1_p4_stock int NOT NULL,
  w1_p5_stock int NOT NULL,
  w1_p6_stock int NOT NULL,
  w1_p7_stock int NOT NULL,
  w1_p8_stock int NOT NULL,
  w1_p9_stock int NOT NULL,

  -- warehouse stock should be lte 32
  CONSTRAINT check_total_stock_w1 CHECK (
    w1_p1_stock + w1_p2_stock + w1_p3_stock + w1_p4_stock + w1_p5_stock +
    w1_p6_stock + w1_p7_stock + w1_p8_stock + w1_p9_stock <= 32
  ),

);

CREATE TABLE IF NOT EXISTS w2_daily_stock(
  day         int NOT NULL primary key references mps(day),

  w2_p1_stock int NOT NULL,
  w2_p2_stock int NOT NULL,
  w2_p3_stock int NOT NULL,
  w2_p4_stock int NOT NULL,
  w2_p5_stock int NOT NULL,
  w2_p6_stock int NOT NULL,
  w2_p7_stock int NOT NULL,
  w2_p8_stock int NOT NULL,
  w2_p9_stock int NOT NULL,

  -- warehouse stock should be lte 32
  CONSTRAINT check_total_stock_w2 CHECK (
    w2_p1_stock + w2_p2_stock + w2_p3_stock + w2_p4_stock + w2_p5_stock +
    w2_p6_stock + w2_p7_stock + w2_p8_stock + w2_p9_stock <= 32
  )
);
