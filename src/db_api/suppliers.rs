use sqlx::postgres::types::PgMoney;
use sqlx::PgConnection;

use super::{RawMaterial, Shipment};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Supplier {
    id: i64,
    raw_material_kind: RawMaterial,
    min_order_quantity: i32,
    unit_price: PgMoney,
    delivery_time: i32,
}

impl Supplier {
    pub fn can_deliver_in(&self, time: i32) -> bool {
        self.delivery_time <= time
    }

    pub fn shipment(&self, order_quantity: i32, due_date: i32) -> Shipment {
        let quantity = order_quantity.max(self.min_order_quantity);
        let cost = quantity as i64 * self.unit_price.0;
        Shipment::new(
            self.id,
            due_date - self.delivery_time,
            quantity,
            cost.into(),
        )
    }

    pub async fn get_by_item_kind(
        kind: RawMaterial,
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<Supplier>> {
        sqlx::query_as!(
            Supplier,
            r#"
            SELECT
                id,
                raw_material_kind as "raw_material_kind: RawMaterial",
                min_order_quantity,
                unit_price,
                delivery_time
            FROM suppliers
            WHERE raw_material_kind = $1
            "#,
            kind as RawMaterial
        )
        .fetch_all(con)
        .await
    }
}
