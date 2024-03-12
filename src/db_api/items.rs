use sqlx::postgres::types::PgMoney;

use super::PieceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "item_status", rename_all = "snake_case")]
pub enum ItemStatus {
    Pending,
    InStock,
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    id: Option<i64>,
    piece_kind: PieceKind,
    order_id: Option<i64>,
    location: Option<String>,
    status: ItemStatus,
    acc_cost: PgMoney,
}

impl Item {
    pub fn new(piece_kind: PieceKind) -> Self {
        Self {
            id: None,
            piece_kind,
            order_id: None,
            location: None,
            status: ItemStatus::Pending,
            acc_cost: PgMoney(0),
        }
    }

    pub fn assign_to_order(self, order_id: i64) -> Self {
        Self {
            order_id: Some(order_id),
            ..self
        }
    }

    pub async fn insert(
        mut self,
        con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<Self> {
        self.id = Some(
            sqlx::query!(
                "INSERT INTO
                items (piece_kind, order_id, location, status, acc_cost)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id",
                self.piece_kind as PieceKind,
                self.order_id,
                self.location,
                self.status as ItemStatus,
                self.acc_cost
            )
            .fetch_one(con)
            .await?
            .id,
        );

        Ok(self)
    }

    pub async fn get_by_id(
        id: i64,
        con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Item,
            r#"SELECT
                id,
                piece_kind as "piece_kind: PieceKind",
                order_id,
                location,
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
                location = $2,
                status = $3,
                acc_cost = $4
            WHERE id = $5"#,
            self.order_id,
            self.location,
            self.status as ItemStatus,
            self.acc_cost,
            self.id
        )
        .execute(con)
        .await?;

        Ok(())
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }
}
