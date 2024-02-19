use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{types::PgMoney, PgQueryResult},
    PgPool,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Money {
    pub cents: i64,
}

impl From<PgMoney> for Money {
    fn from(PgMoney(cents): PgMoney) -> Self {
        Money { cents }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientOrder {
    pub client_name_id: String,
    pub order_number: i64,
    pub work_piece: String,
    pub quantity: i32,
    pub due_date: i32,
    pub late_pen: Money,
    pub early_pen: Money,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteOrder {
    pub client_name_id: String,
    pub order_number: i64,
}

pub async fn fetch_all_orders(
    pool: &PgPool,
) -> Result<Vec<ClientOrder>, sqlx::Error> {
    sqlx::query_as!(ClientOrder, "SELECT * FROM client_orders")
        .fetch_all(pool)
        .await
}

pub async fn fetch_client_orders(
    pool: &PgPool,
    name: &str,
) -> Result<Vec<ClientOrder>, sqlx::Error> {
    sqlx::query_as!(
        ClientOrder,
        "SELECT * FROM client_orders WHERE client_name_id = $1",
        name,
    )
    .fetch_all(pool)
    .await
}

pub async fn place_new_order(
    pool: &PgPool,
    order: &ClientOrder,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO client_orders VALUES ($1, $2, $3, $4, $5, $6, $7)",
        order.client_name_id,
        order.order_number,
        order.work_piece,
        order.quantity,
        order.due_date,
        PgMoney(order.late_pen.cents),
        PgMoney(order.early_pen.cents)
    )
    .execute(pool)
    .await
}

pub async fn update_order(
    pool: &PgPool,
    order: &ClientOrder,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        "UPDATE
                client_orders
            SET
                work_piece = $1,
                quantity = $2,
                due_date = $3,
                late_pen = $4,
                early_pen = $5
            WHERE
                client_name_id = $6 AND order_number = $7",
        order.work_piece,
        order.quantity,
        order.due_date,
        PgMoney(order.late_pen.cents),
        PgMoney(order.early_pen.cents),
        order.client_name_id,
        order.order_number,
    )
    .execute(pool)
    .await
}

pub async fn delete_order(
    pool: &PgPool,
    order: &DeleteOrder,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM client_orders WHERE client_name_id = $1 AND order_number = $2",
        order.client_name_id,
        order.order_number,
    )
    .execute(pool)
    .await
}
