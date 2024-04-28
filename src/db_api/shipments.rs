use sqlx::{postgres::types::PgMoney, PgConnection, PgPool};
use uuid::Uuid;

use super::RawMaterial;

#[derive(Debug)]
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
pub struct ExpectedShippment {
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

    pub async fn arrived(id: i64, date: i32, con: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            WITH item_prices AS (
            SELECT unit_price
            FROM suppliers
            JOIN shipments AS sh
                ON sh.supplier_id = suppliers.id WHERE sh.id = $1
            )
            UPDATE
                items
            SET
                status = 'in_stock',
                warehouse = 'W1',
                acc_cost = (SELECT unit_price FROM item_prices)
            WHERE id IN
            (
                SELECT items.id
                FROM items
                JOIN raw_material_shipments AS rs
                    ON rs.raw_material_id = items.id
                JOIN shipments AS s
                    ON rs.shipment_id = s.id
                WHERE s.id = $1
            )
            "#,
            id,
        )
        .execute(con)
        .await?;

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
    ) -> sqlx::Result<Vec<ExpectedShippment>> {
        sqlx::query_as!(
            ExpectedShippment,
            r#"
            SELECT
                ship.id,
                ship.quantity,
                sup.raw_material_kind as "material_type: RawMaterial"
            FROM shipments AS ship
            JOIN suppliers AS sup ON ship.supplier_id = sup.id
            WHERE request_date + delivery_time = $1
            "#,
            date
        )
        .fetch_all(con)
        .await
    }

    pub async fn get_existing_shippment(
        kind: RawMaterial,
        arrival_date: i32,
        current_date: i32,
        con: &mut PgConnection,
    ) -> sqlx::Result<Option<Shipment>> {
        sqlx::query_as!(
            Shipment,
            r#"
            SELECT
                ship.id,
                ship.supplier_id,
                ship.request_date,
                ship.quantity,
                ship.cost
            FROM shipments as ship
            JOIN suppliers as sup ON ship.supplier_id = sup.id
            WHERE raw_material_kind = $1
                AND request_date > $2
                AND request_date + sup.delivery_time = $3
                AND ship.arrival_date IS NULL
            "#,
            kind as RawMaterial,
            current_date,
            arrival_date
        )
        .fetch_optional(con)
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
            WHERE ship.request_date + sup.delivery_time = $1
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

        tracing::info!("Inserted shippment with id: {}", id);

        Ok(id)
    }

    pub async fn update(&self, con: &mut PgConnection) -> sqlx::Result<i64> {
        let Some(id) = self.id else {
            tracing::error!("Shippment update failed: id not found");
            return Err(sqlx::Error::RowNotFound);
        };

        sqlx::query!(
            r#"
            UPDATE shipments
            SET supplier_id = $1, request_date = $2, quantity = $3, cost = $4
            WHERE id = $5
            "#,
            self.supplier_id,
            self.request_date,
            self.quantity,
            self.cost,
            id
        )
        .execute(con)
        .await?;

        tracing::info!("Updated shippment with id: {}", id);

        Ok(id)
    }

    pub async fn upsert(&self, con: &mut PgConnection) -> sqlx::Result<i64> {
        match self.id.is_some() {
            true => self.update(con).await,
            false => self.insert(con).await,
        }
    }

    pub fn supplier_id(&self) -> i64 {
        self.supplier_id
    }

    pub fn quantity(&self) -> i32 {
        self.quantity
    }

    pub fn add_to_quantity(&mut self, ammount: i32) {
        self.quantity += ammount
    }

    pub fn cost(&self) -> PgMoney {
        self.cost
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }
}

pub struct MaterialShipment {
    raw_material_id: Uuid,
    shippment_id: i64,
}

impl MaterialShipment {
    pub fn new(raw_material_id: Uuid, shippment_id: i64) -> Self {
        Self {
            raw_material_id,
            shippment_id,
        }
    }

    pub async fn insert(&self, con: &mut PgConnection) -> sqlx::Result<()> {
        let res = sqlx::query!(
            r#"
            INSERT INTO raw_material_shipments (raw_material_id, shipment_id)
            VALUES ($1, $2)
            "#,
            self.raw_material_id,
            self.shippment_id,
        )
        .execute(con)
        .await?;

        if res.rows_affected() > 0 {
            tracing::debug!(
            "Inserted raw_material_shipments for material: {} and shipment: {}",
            self.raw_material_id,
            self.shippment_id);
        }

        Ok(())
    }

    pub fn raw_material_id(&self) -> Uuid {
        self.raw_material_id
    }
}
