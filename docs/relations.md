# DB relations

Clients(_id_, name)

Pieces(_id_, Name, Type)

OrderItems(_id_, \#id -> Orders, \#id -> PieceInventory)

PieceInventory(_id_,
\#id -> Pieces,
accumulated_cost,
\#id -> Orders,
\#id -> RawMaterialBatches,
\#id -> Warehouses )

Transformations(_id_, type, \#id -> Pieces, \#id -> Pieces, io_ratio, cost)

BOM(_id_, \#id -> OrderItems, \#id -> Transformations)

Warehouses(_id_, name, capacity, load_time, unload_time)

Suppliers(_id_, name, \#id -> Pieces, min_order, unit_price, delivery_time)

Orders(_id_,
\#id -> Clients,
order_number,
\#id -> Pieces,
quantity,
due_date,
late_penalty,
early_penalty
)

RawMaterialBatches(
_id_,
\#id -> Supplier,
ammount,
total_cost,
purchase_date,
expected_arrival_date,
actual_arrival_date,
)
