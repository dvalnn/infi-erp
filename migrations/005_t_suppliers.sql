CREATE TABLE IF NOT EXISTS suppliers (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    name VARCHAR(255) NOT NULL,
    piece_id BIGINT NOT NULL,
    min_quantity INT NOT NULL,
    unit_price MONEY NOT NULL,
    delivery_time INT NOT NULL,

    FOREIGN KEY(piece_id) REFERENCES pieces(id)
    ON DELETE CASCADE
);

INSERT INTO suppliers (name, piece_id, min_quantity, unit_price, delivery_time)
SELECT 'SupplierA', p.id, 16, 30, 4
FROM pieces p
WHERE p.name = 'P1';

INSERT INTO suppliers (name, piece_id, min_quantity, unit_price, delivery_time)
SELECT 'SupplierA', p.id, 16, 10, 4
FROM pieces p
WHERE p.name = 'P2';

INSERT INTO suppliers (name, piece_id, min_quantity, unit_price, delivery_time)
SELECT 'SupplierB', p.id, 8, 45, 2
FROM pieces p
WHERE p.name = 'P1';

INSERT INTO suppliers (name, piece_id, min_quantity, unit_price, delivery_time)
SELECT 'SupplierB', p.id, 8, 15, 2
FROM pieces p
WHERE p.name = 'P2';

INSERT INTO suppliers (name, piece_id, min_quantity, unit_price, delivery_time)
SELECT 'SupplierC', p.id, 4, 55, 1
FROM pieces p
WHERE p.name = 'P1';

INSERT INTO suppliers (name, piece_id, min_quantity, unit_price, delivery_time)
SELECT 'SupplierC', p.id, 4, 18, 1
FROM pieces p
WHERE p.name = 'P2';


