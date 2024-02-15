-- TODO: Rethink this table design afte more of the
--       sistem funcionality comes online
CREATE TABLE IF NOT EXISTS pieces (
  piece_id         int       GENERATED ALWAYS AS IDENTITY,
  piece_type       varchar            ,
  client_name      varchar            ,
  order_number     bigint             ,
  raw_mat_cost     money     NOT NULL , -- raw material cost
  raw_mat_arr_date int       NOT NULL , -- raw material arrival date
  dispatch_date    int                , -- Query to MES
  total_prod_time  int                , -- Query to MES (in seconds)

  PRIMARY KEY (piece_id),

  CONSTRAINT fk_client_order
    FOREIGN KEY(client_name, order_number) REFERENCES client_orders(client_name_id, order_number)
    ON DELETE SET NULL -- NOTE: may be able to be altered to ON DELETE CASCADE
);
