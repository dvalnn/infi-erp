CREATE TABLE client_orders (
  ClientNameId VARCHAR NOT NULL,
  OrderNumber INT,
  WorkPiece VARCHAR,
  Quantity INT,
  DueDate INT,
  LatePen MONEY,
  EarlyPen MONEY
);
