-- Per day production targets
-- Used to calculate the MPS and the stock projections

CREATE TABLE IF NOT EXISTS daily_production (
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

  -- NOTE: this table may need some way of referencing concrete
  --       piece data
  --
  CONSTRAINT check_total_production CHECK (
    p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 <= 24
  )

);
