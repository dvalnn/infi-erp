use std::collections::HashMap;

use sqlx::postgres::types::PgMoney;
use sqlx::PgConnection;
use uuid::Uuid;

use super::RawMaterial;

#[derive(Debug, Clone)]
pub struct Supplier {
    id: i64,
    raw_material_kind: RawMaterial,
    min_order_quantity: i32,
    unit_price: PgMoney,
    delivery_time: i32,
}

pub struct ShippmentDetails {
    supplier_id: i64,
    quantity: i32,
    cost: PgMoney,
}

impl ShippmentDetails {
    pub fn cost(&self) -> PgMoney {
        self.cost
    }
}

impl Supplier {
    pub fn can_deliver_in(&self, time: i32) -> bool {
        self.delivery_time <= time
    }

    pub fn shippment_details(&self, order_quantity: i32) -> ShippmentDetails {
        let quantity = order_quantity.min(self.min_order_quantity);
        let cost = quantity as i64 * self.unit_price.0;
        ShippmentDetails {
            supplier_id: self.id,
            quantity,
            cost: cost.into(),
        }
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

#[derive(Debug)]
pub struct Shippment {
    id: Option<i64>,
    supplier_id: i64,
    request_date: i32,
    quantity: i32,
    cost: PgMoney,
}

#[derive(Debug, Clone)]
pub struct UnderAllocatedShippment {
    pub id: i64,
    pub extra_quantity: i64,
    pub added: Option<i64>,
}

impl Shippment {
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

    pub async fn get_existing_shippment(
        kind: RawMaterial,
        arrival_date: i32,
        current_date: i32,
        con: &mut PgConnection,
    ) -> sqlx::Result<Option<Shippment>> {
        sqlx::query_as!(
            Shippment,
            r#"
            SELECT
                ship.id,
                ship.supplier_id,
                ship.request_date,
                ship.quantity,
                ship.cost
            FROM shippments as ship
            JOIN suppliers as sup ON ship.supplier_id = sup.id
            WHERE
                raw_material_kind = $1 AND
                request_date > $2 AND
                request_date + sup.delivery_time = $3
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
    ) -> sqlx::Result<Vec<UnderAllocatedShippment>> {
        Ok(sqlx::query!(
            r#"
            SELECT shipp.id, shipp.quantity-COUNT(item.id) as extra_quantity
            FROM shippments as shipp
            JOIN raw_material_shippments as ord ON shipp.id = ord.shippment_id
            JOIN suppliers as sup ON shipp.supplier_id = sup.id
            JOIN items as item ON ord.raw_material_id = item.id
            WHERE shipp.request_date + sup.delivery_time = $1
            AND item.piece_kind = $2
            GROUP BY shipp.id
            HAVING shipp.quantity > COUNT(item.id)
            "#,
            due_date,
            material_kind as RawMaterial
        )
        .fetch_all(con)
        .await?
        .into_iter()
        .map(|row| UnderAllocatedShippment {
            id: row.id,
            extra_quantity: row.extra_quantity.expect("is always Some"),
            added: None,
        })
        .collect())
    }

    pub async fn insert(&self, con: &mut PgConnection) -> sqlx::Result<i64> {
        let id = sqlx::query!(
            r#"
            INSERT INTO shippments (supplier_id, request_date, quantity, cost)
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
                UPDATE shippments
                SET
                    supplier_id = $1,
                    request_date = $2,
                    quantity = $3,
                    cost = $4
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
}

pub struct MaterialShippments {
    raw_material_id: Uuid,
    shippment_id: i64,
}

impl MaterialShippments {
    pub fn new(raw_material_id: Uuid, shippment_id: i64) -> Self {
        Self {
            raw_material_id,
            shippment_id,
        }
    }

    pub async fn insert(&self, con: &mut PgConnection) -> sqlx::Result<()> {
        tracing::info!(
            "Inserting raw_material_shippments for material: {} and shippment: {}",
            self.raw_material_id,
            self.shippment_id);

        sqlx::query!(
            r#"
            INSERT INTO raw_material_shippments (raw_material_id, shippment_id)
            VALUES ($1, $2)
            "#,
            self.raw_material_id,
            self.shippment_id,
        )
        .execute(con)
        .await?;

        tracing::info!(
            "Inserted raw_material_shippments for material: {} and shippment: {}",
            self.raw_material_id,
            self.shippment_id);

        Ok(())
    }
}
