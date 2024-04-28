CREATE TABLE IF NOT EXISTS raw_material_shipments(
  raw_material_id uuid PRIMARY KEY NOT NULL REFERENCES items(id),
  shipment_id bigint NOT NULL REFERENCES shipments(id)
);
