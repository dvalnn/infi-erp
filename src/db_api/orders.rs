use sqlx::{
    postgres::{types::PgMoney, PgQueryResult},
    query,
    types::Uuid,
    PgConnection,
};

use crate::scheduler::{self, Scheduler};

use super::{pieces::FinalPiece, PieceKind};

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Scheduled,
    Producing,
    Completed,
    Delivered,
    Canceled,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Scheduled => write!(f, "scheduled"),
            OrderStatus::Producing => write!(f, "producing"),
            OrderStatus::Completed => write!(f, "completed"),
            OrderStatus::Delivered => write!(f, "delivered"),
            OrderStatus::Canceled => write!(f, "canceled"),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct Order {
    id: Uuid,
    client_id: Uuid,
    number: i32,
    piece: FinalPiece,
    quantity: i32,
    due_date: i32,
    early_penalty: PgMoney,
    late_penalty: PgMoney,

    status: OrderStatus,
    placement_day: i32,
    delivery_day: Option<i32>,
}

#[derive(Debug, serde::Serialize)]
pub struct Delivery {
    id: Uuid,
    piece: FinalPiece,
    quantity: i32,
}

impl Order {
    pub fn new(
        client_id: Uuid,
        number: i32,
        piece: FinalPiece,
        quantity: i32,
        due_date: i32,
        early_penalty: i64,
        late_penalty: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_id,
            number,
            piece,
            quantity,
            due_date,
            early_penalty: PgMoney(early_penalty),
            late_penalty: PgMoney(late_penalty),
            status: OrderStatus::Pending,
            placement_day: 0,
            delivery_day: None,
        }
    }

    pub async fn insert_to_db(
        order: &Order,
        con: &mut PgConnection,
    ) -> sqlx::Result<PgQueryResult> {
        let placement_day = Scheduler::get_date();

        query!(
            r#"INSERT INTO orders (
                id,
                client_id,
                number,
                piece,
                quantity,
                due_date,
                early_penalty,
                late_penalty,
                placement_day
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            order.id,
            order.client_id,
            order.number,
            order.piece as FinalPiece, // cast keeps the quary! macro happy
            order.quantity,
            order.due_date,
            order.early_penalty,
            order.late_penalty,
            placement_day as i32,
        )
        .execute(con)
        .await
    }

    pub async fn get_by_id(
        id: Uuid,
        con: &mut PgConnection,
    ) -> sqlx::Result<Order> {
        //NOTE: the query_as macro dislikes the use of the * wildcard
        //      due to the custom enum types.
        //      So i'm using the query_as function in this scenario.
        //      instead of the macro query_as!
        sqlx::query_as(r#"SELECT * FROM orders WHERE id = $1"#)
            .bind(id)
            .fetch_one(con)
            .await
    }

    pub async fn get_id_by_due_date(
        day: i32,
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<Uuid>> {
        Ok(
            sqlx::query!(r#"SELECT id FROM orders WHERE due_date = $1"#, day)
                .fetch_all(con)
                .await?
                .iter()
                .map(|row| row.id)
                .collect::<Vec<Uuid>>(),
        )
    }

    pub async fn get_by_item_id(
        product_id: Uuid,
        con: &mut PgConnection,
    ) -> sqlx::Result<Option<Order>> {
        sqlx::query_as(
            r#"
            SELECT orders.*
            FROM orders, items
            WHERE
                orders.id = items.order_id
                AND
                items.id = $1
            "#,
        )
        .bind(product_id)
        .fetch_optional(con)
        .await
    }

    pub async fn production_start(
        &self,
        con: &mut PgConnection,
    ) -> sqlx::Result<PgQueryResult> {
        query!(
            r#"UPDATE orders
            SET status = $1
            WHERE id = $2"#,
            OrderStatus::Producing as OrderStatus,
            self.id,
        )
        .execute(con)
        .await
    }

    pub async fn schedule(
        &self,
        delivery_day: i32,
        con: &mut PgConnection,
    ) -> sqlx::Result<PgQueryResult> {
        query!(
            r#"UPDATE orders
            SET delivery_day = $1,
                status = $2
            WHERE id = $3"#,
            delivery_day,
            OrderStatus::Scheduled as OrderStatus,
            self.id,
        )
        .execute(con)
        .await
    }

    pub async fn get_deliveries(
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<Delivery>> {
        sqlx::query_as!(
            Delivery,
            r#"
            SELECT
                id,
                piece as "piece: FinalPiece",
                quantity
            FROM orders
            WHERE status = $1
            "#,
            OrderStatus::Completed as OrderStatus
        )
        .fetch_all(con)
        .await
    }

    pub fn piece(&self) -> PieceKind {
        self.piece.into()
    }

    pub fn quantity(&self) -> i32 {
        self.quantity
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn due_date(&self) -> i32 {
        self.due_date
    }

    pub fn status(&self) -> OrderStatus {
        self.status
    }
}
