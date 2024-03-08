use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::types::PgMoney, query, query_as, types::uuid::Uuid, Executor,
    PgConnection, PgPool, Postgres,
};

use crate::db_api::NotificationChannel as Ntc;

use super::{orders::Order, pieces::FinalPiece};

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
