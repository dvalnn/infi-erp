use sqlx::{postgres::types::PgMoney, PgConnection, PgPool};
use uuid::Uuid;

use super::RawMaterial;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Shipment {
    id: Option<i64>,
    supplier_id: i64,
    request_date: i32,
    quantity: i32,
    cost: PgMoney,
}

#[derive(Debug, Clone)]
pub struct UnderAllocatedShipment {
    pub id: i64,
    pub extra_quantity: i64,
    pub added: Option<i64>,
}

#[derive(Debug)]
pub struct ExpectedShipment {
    pub id: i64,
    pub material_type: RawMaterial,
    pub quantity: i32,
}

impl Shipment {
    pub fn new(
        supplier_id: i64,
        request_date: i32,
        quantity: i32,
        cost: PgMoney,
    ) -> Self {
        Self {
            id: None,
            supplier_id,
            request_date,
            quantity,
            cost,
        }
    }

    pub async fn delete(id: i64, con: &mut PgConnection) -> sqlx::Result<()> {
        sqlx::query!(r#"DELETE FROM shipments WHERE shipments.id = $1"#, id)
            .execute(con)
            .await?;

        Ok(())
    }

    pub async fn arrived(id: i64, date: i32, con: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            UPDATE shipments
            SET arrival_date = $1
            WHERE id = $2
            "#,
            date,
            id
        )
        .execute(con)
        .await?;

        Ok(())
    }

    pub async fn get_expected_for_arrival(
        date: i32,
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<ExpectedShipment>> {
        sqlx::query_as!(
            ExpectedShipment,
            r#"
            SELECT
                ship.id,
                ship.quantity,
                sup.raw_material_kind as "material_type: RawMaterial"
            FROM shipments AS ship
            JOIN suppliers AS sup ON ship.supplier_id = sup.id
            WHERE request_date + delivery_time <= $1
              AND arrival_date IS NULL
            "#,
            date
        )
        .fetch_all(con)
        .await
    }

    pub async fn get_under_allocated(
        due_date: i32,
        material_kind: RawMaterial,
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<UnderAllocatedShipment>> {
        Ok(sqlx::query!(
            r#"
            SELECT ship.id, ship.quantity-COUNT(item.id) as extra_quantity
            FROM shipments as ship
            JOIN raw_material_shipments as ord ON ship.id = ord.shipment_id
            JOIN suppliers as sup ON ship.supplier_id = sup.id
            JOIN items as item ON ord.raw_material_id = item.id
            WHERE ship.request_date + sup.delivery_time <= $1
                AND item.piece_kind = $2
                AND ship.arrival_date IS NULL
            GROUP BY ship.id
            HAVING ship.quantity > COUNT(item.id)
            "#,
            due_date,
            material_kind as RawMaterial
        )
        .fetch_all(con)
        .await?
        .into_iter()
        .map(|row| UnderAllocatedShipment {
            id: row.id,
            extra_quantity: row.extra_quantity.expect("is always Some"),
            added: None,
        })
        .collect())
    }

    pub async fn insert(&self, con: &mut PgConnection) -> sqlx::Result<i64> {
        let id = sqlx::query!(
            r#"
            INSERT INTO shipments (supplier_id, request_date, quantity, cost)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            self.supplier_id,
            self.request_date,
            self.quantity,
            self.cost
        )
        .fetch_one(con)
        .await?
        .id;

        tracing::info!("Inserted shipment with id: {}", id);

        Ok(id)
    }

    pub fn cost(&self) -> PgMoney {
        self.cost
    }
}

pub struct MaterialShipment {
    raw_material_id: Uuid,
    shipment_id: i64,
}

impl MaterialShipment {
    pub fn new(raw_material_id: Uuid, shipment_id: i64) -> Self {
        Self {
            raw_material_id,
            shipment_id,
        }
    }

    pub async fn insert(&self, con: &mut PgConnection) -> sqlx::Result<()> {
        let res = sqlx::query!(
            r#"
            INSERT INTO raw_material_shipments (raw_material_id, shipment_id)
            VALUES ($1, $2)
            "#,
            self.raw_material_id,
            self.shipment_id,
        )
        .execute(con)
        .await?;

        if res.rows_affected() > 0 {
            tracing::debug!(
            "Inserted raw_material_shipments for material: {} and shipment: {}",
            self.raw_material_id,
            self.shipment_id);
        }

        Ok(())
    }

    pub fn raw_material_id(&self) -> Uuid {
        self.raw_material_id
    }
}
