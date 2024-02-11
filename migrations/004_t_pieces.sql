CREATE TABLE IF NOT EXISTS pieces (
  id               bigserial NOT NULL primary key,
  piece_type       varchar   NOT NULL ,
  client_order     int       NOT NULL references client_orders(order_number),
  raw_mat_cost     money     NOT NULL , -- raw material cost
  raw_mat_arr_date int       NOT NULL , -- raw material arrival date
  dispatch_date    int                , -- Query to MES
  total_prod_time  int                  -- Query to MES (in seconds)
);
