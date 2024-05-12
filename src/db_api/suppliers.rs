use std::collections::HashMap;

use sqlx::postgres::types::PgMoney;
use sqlx::PgConnection;
use uuid::Uuid;

use super::{RawMaterial, Shipment};

#[derive(Debug, Clone)]
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

    pub async fn get_by_id(
        id: i64,
        con: &mut PgConnection,
    ) -> sqlx::Result<Supplier> {
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
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(con)
        .await
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

    pub async fn get_compatible(
        kind: RawMaterial,
        time: i32,
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
            WHERE raw_material_kind = $1 AND delivery_time <= $2
            "#,
            kind as RawMaterial,
            time
        )
        .fetch_all(con)
        .await
    }

    pub fn delivery_time(&self) -> i32 {
        self.delivery_time
    }

    pub fn unit_price(&self) -> PgMoney {
        self.unit_price
    }

    pub fn min_order_quantity(&self) -> i32 {
        self.min_order_quantity
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}
