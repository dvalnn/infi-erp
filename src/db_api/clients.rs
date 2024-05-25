use sqlx::{types::uuid::Uuid, PgConnection, PgPool};

use crate::db_api::NotificationChannel as Ntc;

use super::{orders::Order, pieces::FinalPiece};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Client {
    id: Uuid,
    name: String,
}

impl Client {
    pub async fn query_by_name(
        name: &str,
        con: &mut PgConnection,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Client,
            r#"SELECT * FROM clients WHERE name = $1"#,
            name
        )
        .fetch_optional(con)
        .await
    }

    pub async fn insert_to_db(
        name: &str,
        con: &mut PgConnection,
    ) -> sqlx::Result<Uuid> {
        Ok(sqlx::query!(
            "INSERT INTO clients (name) VALUES ($1) RETURNING id",
            name
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
    pub work_piece: FinalPiece,
    pub quantity: i32,
    pub due_date: i32,
    pub late_penalty: i64,
    pub early_penalty: i64,
}

impl ClientOrder {
    #[allow(dead_code)]
    pub fn new(
        client_name: String,
        order_number: i32,
        work_piece: FinalPiece,
        quantity: i32,
        due_date: i32,
        late_penalty: i64,
        early_penalty: i64,
    ) -> Self {
        Self {
            client_name,
            order_number,
            work_piece,
            quantity,
            due_date,
            late_penalty,
            early_penalty,
        }
    }

    pub async fn insert_to_db(&self, pool: &PgPool) -> sqlx::Result<Uuid> {
        let mut tx = pool.begin().await?;

        // check if client exists in db
        // if not insert client and then insert order
        // if client exists only insert order
        let client_id = match Client::query_by_name(&self.client_name, &mut tx)
            .await?
        {
            Some(c) => c.id,
            None => {
                tracing::info!("Inserting new client '{}'", &self.client_name);
                Client::insert_to_db(&self.client_name, &mut tx).await?
            }
        };

        let new_order = Order::new(
            client_id,
            self.order_number,
            self.work_piece,
            self.quantity,
            self.due_date,
            self.early_penalty,
            self.late_penalty,
        );
        Order::insert_to_db(&new_order, &mut tx).await?;

        Ntc::notify(Ntc::NewOrder, &new_order.id().to_string(), &mut tx)
            .await?;
        tx.commit().await?;

        tracing::info!("Inserted new order id: {}", new_order.id());

        Ok(new_order.id())
    }
}
