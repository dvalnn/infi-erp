use sqlx::postgres::{types::PgMoney, PgQueryResult};
use uuid::Uuid;

use super::PieceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "item_status", rename_all = "snake_case")]
pub enum ItemStatus {
    Pending,
    InStock,
    InTransit,
    Delivered,
    Consumed,
}

impl std::fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ItemStatus::Pending => write!(f, "pending"),
            ItemStatus::InStock => write!(f, "in_stock"),
            ItemStatus::Delivered => write!(f, "delivered"),
            ItemStatus::Consumed => write!(f, "consumed"),
            ItemStatus::InTransit => write!(f, "in_transit"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    id: Uuid,
    piece_kind: PieceKind,
    order_id: Option<Uuid>,
    warehouse: Option<String>,
    status: ItemStatus,
    acc_cost: PgMoney,
}

impl Item {
    pub fn new(piece_kind: PieceKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            piece_kind,
            order_id: None,
            warehouse: None,
            status: ItemStatus::Pending,
            acc_cost: PgMoney(0),
        }
    }

    pub fn set_order(mut self, order_id: Option<Uuid>) -> Self {
        self.order_id = order_id;
        self
    }

    pub fn produce(mut self, cost: PgMoney) -> anyhow::Result<Self> {
        if self.status != ItemStatus::Pending {
            anyhow::bail!(format!(
                "Item {} is {}, cannot produce",
                self.id, self.status
            ));
        }

        self.status = ItemStatus::InTransit;
        self.acc_cost = cost;
        Ok(self)
    }

    pub fn consume(mut self) -> anyhow::Result<Self> {
        if self.status != ItemStatus::InTransit {
            anyhow::bail!(format!(
                "Item {} is {}, cannot consume",
                self.id, self.status
            ));
        }

        self.status = ItemStatus::Consumed;
        self.warehouse = None;
        Ok(self)
    }

    pub fn enter_warehouse(
        mut self,
        warehouse: impl ToString,
    ) -> anyhow::Result<Self> {
        if self.status != ItemStatus::InTransit {
            anyhow::bail!(format!(
                "Item {} is {}, cannot enter warehouse",
                self.id, self.status
            ));
        }

        self.status = ItemStatus::InStock;
        self.warehouse = Some(warehouse.to_string());
        Ok(self)
    }

    pub fn exit_warehouse(mut self) -> anyhow::Result<Self> {
        if self.status != ItemStatus::InStock {
            anyhow::bail!(format!(
                "Item {} is {}, cannot exit warehouse",
                self.id, self.status
            ));
        }

        self.status = ItemStatus::InTransit;
        self.warehouse = None;
        Ok(self)
    }

    pub fn get_cost(&self) -> PgMoney {
        self.acc_cost
    }

    pub async fn insert(
        &self,
        con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "INSERT INTO
                items (id, piece_kind, order_id, warehouse, status, acc_cost)
                VALUES ($1, $2, $3, $4, $5, $6)",
            self.id,
            self.piece_kind as PieceKind,
            self.order_id,
            self.warehouse,
            self.status as ItemStatus,
            self.acc_cost
        )
        .execute(con)
        .await
    }

    pub async fn get_by_id(
        id: Uuid,
        con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Item,
            r#"SELECT
                id,
                piece_kind as "piece_kind: PieceKind",
                order_id,
                warehouse,
                status as "status: ItemStatus",
                acc_cost
            FROM items WHERE id = $1"#,
            id
        )
        .fetch_one(con)
        .await
    }

    pub async fn current_stock(
        con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<i64> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as quantity
            FROM items
            WHERE items.status = $1
            "#,
            ItemStatus::InStock as ItemStatus,
        )
        .fetch_one(con)
        .await?;
        Ok(row.quantity.unwrap_or(0))
    }

    pub async fn get_stock_prevision(
        _day: i32,
        _con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<i64> {
        //TODO: total stock prevision for day is current stock + arrivals - departures
        todo!()
    }

    pub async fn update(
        &self,
        con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"UPDATE items
            SET
                order_id = $1,
                warehouse = $2,
                status = $3,
                acc_cost = $4
            WHERE id = $5"#,
            self.order_id,
            self.warehouse,
            self.status as ItemStatus,
            self.acc_cost,
            self.id
        )
        .execute(con)
        .await?;

        Ok(())
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn order_id(&self) -> Option<Uuid> {
        self.order_id
    }
}
