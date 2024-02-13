-- Per day production targets
-- Used to calculate the MPS and the stock projections

CREATE TABLE IF NOT EXISTS daily_production (
  day int NOT NULL primary key,

  p3  int ,
  p4  int ,
  p5  int ,
  p6  int ,
  p7  int ,
  p8  int ,
  p9  int ,

  -- NOTE: this table may need some way of referencing concrete
  --       piece data
  --
  CONSTRAINT check_total_production CHECK (
    p3 + p4 + p5 + p6 + p7 + p8 + p9 <= 24
  )

);
