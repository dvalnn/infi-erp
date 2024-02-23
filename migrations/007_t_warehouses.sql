CREATE TABLE IF NOT EXISTS warehouses(
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    name VARCHAR NOT NULL,
    capacity INT NOT NULL,
    load_time INT DEFAULT 0,
    unload_time INT DEFAULT 0,

    UNIQUE(name)
);

INSERT INTO warehouses (name, capacity, load_time, unload_time) VALUES ('W1', 32, 3, 3);
INSERT INTO warehouses (name, capacity, load_time, unload_time) VALUES ('W2', 32, 3, 3);
