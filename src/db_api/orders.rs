use sqlx::{postgres::types::PgMoney, query, types::Uuid, PgConnection};

use super::{pieces::FinalPiece, PieceKind};

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Scheduled,
    Delivered,
    Canceled,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Scheduled => write!(f, "scheduled"),
            OrderStatus::Delivered => write!(f, "delivered"),
            OrderStatus::Canceled => write!(f, "canceled"),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct Order {
    id: Option<i64>,

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
            id: None,
            client_id,
            number,
            piece,
            quantity,
            due_date,
            early_penalty: PgMoney(early_penalty),
            late_penalty: PgMoney(late_penalty),
            status: OrderStatus::Pending,
            placement_day: 1, //TODO: get current day
            delivery_day: None,
        }
    }

    pub async fn insert(
        order: Order,
        con: &mut PgConnection,
    ) -> sqlx::Result<i64> {
        Ok(query!(
            r#"INSERT INTO orders (
                client_id,
                number,
                piece,
                quantity,
                due_date,
                early_penalty,
                late_penalty,
                placement_day
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            order.client_id,
            order.number,
            order.piece as FinalPiece, // makes the macro happy
            order.quantity,
            order.due_date,
            order.early_penalty,
            order.late_penalty,
            order.placement_day,
        )
        .fetch_one(con)
        .await?
        .id)
    }

    pub async fn get_by_id(
        id: i64,
        con: &mut PgConnection,
    ) -> sqlx::Result<Order> {
        //NOTE: the query_as macro dislikes the use of the * wildcard
        //      due to the custom enum types.
        //      So i'm using the query_as function in this scenario.
        let order: Order =
            sqlx::query_as(r#"SELECT * FROM orders WHERE id = $1"#)
                .bind(id)
                .fetch_one(con)
                .await?;

        Ok(order)
    }

    pub fn piece(&self) -> PieceKind {
        self.piece.into()
    }

    pub fn quantity(&self) -> i32 {
        self.quantity
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }
}
