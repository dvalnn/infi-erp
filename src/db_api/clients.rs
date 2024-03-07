use sqlx::{
    postgres::types::PgMoney, query, query_as, types::uuid::Uuid, Executor,
    PgConnection, PgPool, Postgres,
};

use crate::db_api::NotificationChannel as Ntc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkPiece {
    P5,
    P6,
    P7,
    P9,
}

impl std::fmt::Display for WorkPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkPiece::P5 => write!(f, "P5"),
            WorkPiece::P6 => write!(f, "P6"),
            WorkPiece::P7 => write!(f, "P7"),
            WorkPiece::P9 => write!(f, "P9"),
        }
    }
}

impl WorkPiece {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "P5" => Some(WorkPiece::P5),
            "P6" => Some(WorkPiece::P6),
            "P7" => Some(WorkPiece::P7),
            "P9" => Some(WorkPiece::P9),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Client {
    id: Option<Uuid>,
    name: String,
}

impl Client {
    fn new(name: String) -> Self {
        Self { id: None, name }
    }

    pub async fn query_by_name(
        name: &str,
        con: &mut PgConnection,
    ) -> sqlx::Result<Option<Self>> {
        query_as!(Client, r#"SELECT * FROM clients WHERE name = $1"#, name)
            .fetch_optional(con)
            .await
    }

    pub async fn insert(
        name: &str,
        con: &mut PgConnection,
    ) -> sqlx::Result<Uuid> {
        Ok(
            query!("INSERT INTO clients (name) VALUES ($1) RETURNING id", name)
                .fetch_one(con)
                .await?
                .id,
        )
    }
}

#[derive(Debug)]
pub struct Order {
    id: Option<u64>,
    client_id: Uuid,
    number: i32,
    piece: WorkPiece,
    quantity: i32,
    due_date: i32,
    early_penalty: PgMoney,
    late_penalty: PgMoney,
    placement_day: i32,
}

impl Order {
    pub fn new(
        client_id: Uuid,
        number: i32,
        piece: WorkPiece,
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
            placement_day: 1, //TODO: get current date
        }
    }

    pub async fn insert(
        order: Order,
        con: &mut PgConnection,
    ) -> sqlx::Result<i64> {
        Ok(query!(
            "INSERT INTO orders (
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
            ",
            order.client_id,
            order.number,
            order.piece.to_string(),
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
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClientOrder {
    pub client_name: String,
    pub order_number: i32,
    pub work_piece: WorkPiece,
    pub quantity: i32,
    pub due_date: i32,
    pub late_penalty: i64,
    pub early_penalty: i64,
}

impl ClientOrder {
    pub async fn insert_to_db(&self, pool: &PgPool) -> sqlx::Result<i64> {
        let mut tx = pool.begin().await?;

        // check if client exists in db
        // if not insert client and then insert order
        // if client exists only insert order
        let client_id = match Client::query_by_name(&self.client_name, &mut tx)
            .await?
        {
            Some(c) => c.id.expect("Existing client should always have uuid"),
            None => {
                tracing::info!("Inserting new client '{}'", &self.client_name);
                Client::insert(&self.client_name, &mut tx).await?
            }
        };

        let id = Order::insert(
            Order::new(
                client_id,
                self.order_number,
                self.work_piece,
                self.quantity,
                self.due_date,
                self.early_penalty,
                self.late_penalty,
            ),
            &mut tx,
        )
        .await?;

        Ntc::notify(Ntc::NewOrder, &id.to_string(), &mut tx).await?;
        tx.commit().await?;

        tracing::info!("Inserted new order id: {}", id);

        Ok(id)
    }
}
