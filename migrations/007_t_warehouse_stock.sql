-- Per day stock projections for each warehouse.
-- Used to calculate the MPS
-- Dependent on the the production targets

-- NOTE: W1 only stores raw and intermediate products
CREATE TABLE IF NOT EXISTS w1_daily_stock(
  day int NOT NULL primary key,

  p1  int , -- raw material
  p2  int , -- raw material
  p3  int , -- intermediate
  p4  int , -- intermediate
  p8  int , -- intermediate

  -- warehouse stock should be lte 32
  CONSTRAINT check_total CHECK (
    p1 + p2 + p3 + p4 + p8 <= 32
  )
);

-- NOTE: W2 ideally only stores final products
--       but can also hold intermediate pieces
CREATE TABLE IF NOT EXISTS w2_daily_stock(
  day int NOT NULL primary key,

  p3  int , -- intermediate
  p4  int , -- intermediate
  p5  int , -- final
  p6  int , -- final
  p7  int , -- final
  p8  int , -- intermediate
  p9  int , -- final

  -- warehouse stock should be lte 32
  CONSTRAINT check_total CHECK (
    p3 + p4 + p5 + p6 + p7 + p8 + p9 <= 32
  )
);
