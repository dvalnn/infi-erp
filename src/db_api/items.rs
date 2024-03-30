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
    production_line: Option<String>,
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
            production_line: None,
            status: ItemStatus::Pending,
            acc_cost: PgMoney(0),
        }
    }

    pub fn set_order(mut self, order_id: Option<Uuid>) -> Self {
        self.order_id = order_id;
        self
    }

    pub fn produce(
        mut self,
        cost: PgMoney,
        production_line: impl ToString,
    ) -> anyhow::Result<Self> {
        if self.status != ItemStatus::Pending {
            anyhow::bail!(format!("Item is {}, cannot produce", self.status));
        }

        self.status = ItemStatus::InTransit;
        self.production_line = Some(production_line.to_string());
        self.acc_cost = cost;
        Ok(self)
    }

    pub fn consume(
        mut self,
        production_line: impl ToString,
    ) -> anyhow::Result<Self> {
        if self.status != ItemStatus::InTransit {
            anyhow::bail!(format!("Item is {}, cannot consume", self.status));
        }

        self.status = ItemStatus::Consumed;
        self.warehouse = None;
        self.production_line = Some(production_line.to_string());
        Ok(self)
    }

    pub fn enter_warehouse(
        mut self,
        warehouse: impl ToString,
    ) -> anyhow::Result<Self> {
        if self.status != ItemStatus::InTransit {
            anyhow::bail!(format!(
                "Item is {}, cannot enter warehouse",
                self.status
            ));
        }

        self.status = ItemStatus::InStock;
        self.warehouse = Some(warehouse.to_string());
        self.production_line = None;
        Ok(self)
    }

    pub fn exit_warehouse(
        mut self,
        production_line: impl ToString,
    ) -> anyhow::Result<Self> {
        if self.status != ItemStatus::InStock {
            //TODO: remove this log and uncomment the bail
            // after raw material requests are implemented
            tracing::warn!("Allowing {} item to leave warehouse", self.status);
            // anyhow::bail!(format!(
            //     "Item is {}, cannot exit warehouse",
            //     self.status
            // ));
        }

        self.status = ItemStatus::InTransit;
        self.warehouse = None;
        self.production_line = Some(production_line.to_string());
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
                items (id, piece_kind, order_id, warehouse, production_line, status, acc_cost)
                VALUES ($1, $2, $3, $4, $5, $6, $7)",
            self.id,
            self.piece_kind as PieceKind,
            self.order_id,
            self.warehouse,
            self.production_line,
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
                production_line,
                status as "status: ItemStatus",
                acc_cost
            FROM items WHERE id = $1"#,
            id
        )
        .fetch_one(con)
        .await
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
                production_line = $3,
                status = $4,
                acc_cost = $5
            WHERE id = $6"#,
            self.order_id,
            self.warehouse,
            self.production_line,
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
