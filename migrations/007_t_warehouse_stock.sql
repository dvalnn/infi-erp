-- Per day stock projections for each warehouse.
-- Used to calculate the MPS
-- Dependent on the the production targets

CREATE TABLE IF NOT EXISTS w1_daily_stock(
  day int NOT NULL primary key,

  p1  int NOT NULL,
  p2  int NOT NULL,
  p3  int NOT NULL,
  p4  int NOT NULL,
  p5  int NOT NULL,
  p6  int NOT NULL,
  p7  int NOT NULL,
  p8  int NOT NULL,
  p9  int NOT NULL,

  -- warehouse stock should be lte 32
  CONSTRAINT check_total CHECK (
    p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 <= 32
  )
);

CREATE TABLE IF NOT EXISTS w2_daily_stock(
  day int NOT NULL primary key,

  p1  int NOT NULL,
  p2  int NOT NULL,
  p3  int NOT NULL,
  p4  int NOT NULL,
  p5  int NOT NULL,
  p6  int NOT NULL,
  p7  int NOT NULL,
  p8  int NOT NULL,
  p9  int NOT NULL,

  -- warehouse stock should be lte 32
  CONSTRAINT check_total CHECK (
    p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 <= 32
  )
);
