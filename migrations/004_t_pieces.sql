CREATE TABLE IF NOT EXISTS pieces (
  piece_id            bigserial   NOT NULL primary key,
  raw_mat_arr_date    int         NOT NULL, -- raw material arrival date
  raw_mat_cost        money       NOT NULL, -- raw material cost
  dispatch_date       int         ,
  total_prod_time     int                   -- total production time (seconds)
);
