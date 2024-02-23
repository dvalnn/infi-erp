use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{types::PgMoney, PgQueryResult},
    PgPool,
};

type MyExecutor<'this> = &'this mut sqlx::PgConnection;

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
pub struct OrderDetails {
    pub client_name: String,
    pub order_number: i32,
    pub piece_name: String,
    pub quantity: i32,
    pub due_date: i32,
    pub late_penalty: Money,
    pub early_penalty: Money,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteOrder {
    pub client_name: String,
    pub order_number: i32,
}

pub async fn fetch_all_orders(
    pool: &PgPool,
) -> Result<Vec<OrderDetails>, sqlx::Error> {
    sqlx::query_as!(
        OrderDetails,
        "
            SELECT
                c.name AS client_name,
                o.order_number,
                p.name AS piece_name,
                o.quantity,
                o.due_date,
                o.early_penalty,
                o.late_penalty
            FROM orders o
            INNER JOIN clients c ON c.id = o.client_id
            INNER JOIN pieces p ON p.id = o.work_piece;
        "
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_client_orders(
    pool: &PgPool,
    name: &str,
) -> Result<Vec<OrderDetails>, sqlx::Error> {
    sqlx::query_as!(
        OrderDetails,
        "
            SELECT
                c.name AS client_name,
                o.order_number,
                p.name AS piece_name,
                o.quantity,
                o.due_date,
                o.early_penalty,
                o.late_penalty
            FROM orders o
            INNER JOIN clients c ON c.id = o.client_id
            INNER JOIN pieces p ON p.id = o.work_piece
            WHERE c.name = $1;
        ",
        name
    )
    .fetch_all(pool)
    .await
}

pub async fn place_new_order(
    pool: &PgPool,
    order: &OrderDetails,
) -> Result<PgQueryResult, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let piece_id = get_piece_id(&order.piece_name, &mut tx).await?;

    let client_id = match get_piece_id(&order.client_name, &mut tx).await {
        Ok(id) => id,
        Err(_) => {
            sqlx::query!(
                "INSERT INTO clients(name) VALUES($1) RETURNING id",
                order.client_name
            )
            .fetch_one(&mut tx as MyExecutor)
            .await?
            .id
        }
    };

    let result = sqlx::query!(
        "INSERT INTO orders (
            work_piece,
            client_id,
            order_number,
            quantity,
            due_date,
            late_penalty,
            early_penalty)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7);
        ",
        piece_id,
        client_id,
        order.order_number,
        order.quantity,
        order.due_date,
        PgMoney(order.late_penalty.cents),
        PgMoney(order.early_penalty.cents)
    )
    .execute(&mut tx as MyExecutor)
    .await?;

    tx.commit().await?;

    Ok(result)
}

pub async fn update_order(
    pool: &PgPool,
    order: &OrderDetails,
) -> Result<PgQueryResult, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let client_id = get_client_id(&order.client_name, &mut tx).await?;
    let piece_id = get_piece_id(&order.piece_name, &mut tx).await?;

    let result = sqlx::query!(
        "UPDATE
                orders
            SET
                work_piece = $1,
                quantity = $2,
                due_date = $3,
                late_penalty = $4,
                early_penalty= $5
            WHERE
                client_id = $6 AND order_number = $7",
        piece_id,
        order.quantity,
        order.due_date,
        PgMoney(order.late_penalty.cents),
        PgMoney(order.early_penalty.cents),
        client_id,
        order.order_number,
    )
    .execute(&mut tx as MyExecutor)
    .await?;

    tx.commit().await?;

    Ok(result)
}

pub async fn delete_order(
    pool: &PgPool,
    order: &DeleteOrder,
) -> Result<PgQueryResult, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let client_id = get_client_id(&order.client_name, &mut tx).await?;

    let result = sqlx::query!(
        "DELETE FROM orders WHERE client_id = $1 AND order_number = $2",
        client_id,
        order.order_number,
    )
    .execute(pool)
    .await?;

    tx.commit().await?;

    Ok(result)
}

async fn get_client_id(
    name: &str,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<i64, sqlx::Error> {
    let client_id =
        sqlx::query!("SELECT id FROM clients WHERE name = $1", name)
            .fetch_one(tx as MyExecutor)
            .await?
            .id;
    Ok(client_id)
}

async fn get_piece_id(
    name: &str,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<i64, sqlx::Error> {
    let piece_id = sqlx::query!("SELECT id FROM pieces WHERE name = $1 ", name)
        .fetch_one(tx as MyExecutor)
        .await?
        .id;
    Ok(piece_id)
}
