CREATE TABLE IF NOT EXISTS recipes (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  material_kind char(2) NOT NULL REFERENCES pieces(code),
  product_kind char(2) NOT NULL REFERENCES pieces(code),
  tool char(2) NOT NULL REFERENCES tools(code),
  operation_time int NOT NULL,

  CHECK (material_kind <> product_kind),
  UNIQUE (material_kind, product_kind, tool)
);

INSERT INTO recipes (material_kind, product_kind, tool, operation_time)
VALUES ('P1', 'P3', 'T1', 45),

       ('P3', 'P4', 'T2', 15),
       ('P3', 'P4', 'T3', 25),

       ('P4', 'P5', 'T4', 25),
       ('P4', 'P6', 'T2', 25),
       ('P4', 'P7', 'T3', 15),

       ('P2', 'P8', 'T1', 45),

       ('P8', 'P7', 'T6', 15),
       ('P8', 'P9', 'T5', 45);
