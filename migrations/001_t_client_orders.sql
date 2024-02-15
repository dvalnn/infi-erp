CREATE TABLE IF NOT EXISTS client_orders (
  client_name_id  varchar       ,
  order_number    bigint        ,
  work_piece      varchar       NOT NULL,
  quantity        int           NOT NULL,
  due_date        int           NOT NULL,
  late_pen        money         NOT NULL,
  early_pen       money         NOT NULL,

  PRIMARY KEY (client_name_id, order_number)
);
