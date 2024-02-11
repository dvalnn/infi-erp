-- Per day production targets
-- Used to calculate the MPS and the stock projections

CREATE TABLE IF NOT EXISTS daily_production (
  day           int NOT NULL primary key,

  p1_production int NOT NULL,
  p2_production int NOT NULL,
  p3_production int NOT NULL,
  p4_production int NOT NULL,
  p5_production int NOT NULL,
  p6_production int NOT NULL,
  p7_production int NOT NULL,
  p8_production int NOT NULL,
  p9_production int NOT NULL,

  -- NOTE: this table may need some way of referencing concrete
  --       piece data

  CONSTRAINT check_total_production CHECK (
    p1_production + p2_production + p3_production + p4_production +
    p5_production + p6_production + p7_production + p8_production +
    p9_production <= 24
  ),

);
