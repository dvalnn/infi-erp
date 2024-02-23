CREATE TABLE IF NOT EXISTS supply_batches(
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    supplier BIGINT NOT NULL,
    ammount INT NOT NULL,
    total_cost MONEY NOT NULL,
    purchase_date INT NOT NULL,
    expected_arrival_date INT NOT NULL,
    actual_arrival_date INT,

    FOREIGN KEY(supplier) REFERENCES suppliers(id)
    ON DELETE CASCADE
);
