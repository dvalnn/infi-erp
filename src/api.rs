use poem::{
    error::InternalServerError,
    web::{Data, Path},
    Result,
};

use poem_openapi::{
    payload::{Json, PlainText},
    Object, OpenApi,
};

use sqlx::postgres::{types::PgMoney, PgPool};

#[derive(Object)]
struct Money {
    cents: i64,
}

impl From<PgMoney> for Money {
    fn from(PgMoney(cents): PgMoney) -> Self {
        Money { cents }
    }
}

#[derive(Object)]
struct ClientOrderPayload {
    client_name_id: String,
    order_number: i64,
    work_piece: String,
    quantity: i32,
    due_date: i32,
    late_pen: Money,
    early_pen: Money,
}

#[derive(Object)]
struct DeleteOrderPayload {
    client_name_id: String,
    order_number: i64,
}

type ClientOrderResponse = Result<Json<Vec<ClientOrderPayload>>>;

pub struct ClientOrderApi;

#[OpenApi]
impl ClientOrderApi {
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> PlainText<&'static str> {
        PlainText("Hello, world!")
    }

    #[oai(path = "/orders/:name", method = "get")]
    async fn get_from_client(
        &self,
        Path(name): Path<String>,
        Data(pool): Data<&PgPool>,
    ) -> ClientOrderResponse {
        tracing::info!("Fetching orders for: {}", name);

        let orders = sqlx::query_as!(
            ClientOrderPayload,
            "SELECT * FROM client_orders WHERE client_name_id = $1",
            name
        )
        .fetch_all(pool)
        .await
        .map_err(InternalServerError)?;

        match orders.is_empty() {
            true => Err(poem::error::NotFoundError.into()),
            false => Ok(Json(orders)),
        }
    }

    #[oai(path = "/orders", method = "get")]
    async fn get_all(&self, Data(pool): Data<&PgPool>) -> ClientOrderResponse {
        let query =
            sqlx::query_as!(ClientOrderPayload, "SELECT * FROM client_orders");

        let orders =
            query.fetch_all(pool).await.map_err(InternalServerError)?;

        if orders.is_empty() {
            return Err(poem::error::NotFoundError.into());
        }

        Ok(Json(orders))
    }

    #[oai(path = "/orders", method = "post")]
    async fn place_order(
        &self,
        Data(pool): Data<&PgPool>,
        Json(order): Json<ClientOrderPayload>,
    ) -> Result<()> {
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
        .map_err(InternalServerError)?;

        Ok(())
    }

    #[oai(path = "/orders", method = "put")]
    async fn update_order(
        &self,
        Data(pool): Data<&PgPool>,
        Json(order): Json<ClientOrderPayload>,
    ) -> Result<()> {
        tracing::info!(
            "Updating order: {} {}",
            order.client_name_id,
            order.order_number
        );

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
            order.order_number
        )
        .execute(pool)
        .await
        .map_err(InternalServerError)?;

        Ok(())
    }

    #[oai(path = "/orders", method = "delete")]
    async fn delete_order(
        &self,
        Data(pool): Data<&PgPool>,
        Json(order): Json<DeleteOrderPayload>,
    ) -> Result<()> {
        tracing::info!(
            "Deleting order: {} - {}",
            order.client_name_id,
            order.order_number
        );

        sqlx::query!(
            "DELETE FROM client_orders \
             WHERE client_name_id = $1 AND order_number = $2",
            order.client_name_id,
            order.order_number
        )
        .execute(pool)
        .await
        .map_err(InternalServerError)?;

        Ok(())
    }
}
