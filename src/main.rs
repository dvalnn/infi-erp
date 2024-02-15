#![forbid(unsafe_code)]
#![allow(dead_code, unused_variables)]

mod xml;

use color_eyre::eyre::Result;

use poem::{
    error::InternalServerError,
    listener::TcpListener,
    web::{Data, Path},
    EndpointExt, Route, Server,
};

use poem_openapi::{
    payload::{Json, PlainText},
    Object, OpenApi, OpenApiService,
};

use sqlx::{
    error::BoxDynError,
    postgres::{types::PgMoney, PgPool},
};

#[derive(Object)]
struct Money {
    cents: i64,
}

impl From<PgMoney> for Money {
    fn from(value: PgMoney) -> Self {
        Money { cents: value.0 }
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

type ClientOrderResponse = poem::Result<Json<Vec<ClientOrderPayload>>>;

struct ClientOrderApi;

#[OpenApi]
impl ClientOrderApi {
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> PlainText<&'static str> {
        PlainText("Hello, world!")
    }

    #[oai(path = "/orders/:name", method = "get")]
    async fn get_from_client(
        &self,
        name: Path<String>,
        pool: Data<&PgPool>,
    ) -> ClientOrderResponse {
        tracing::info!("Fetching orders for: {}", name.0);

        let orders = sqlx::query_as!(
            ClientOrderPayload,
            "SELECT * FROM client_orders WHERE client_name_id = $1",
            name.0
        )
        .fetch_all(pool.0)
        .await
        .map_err(InternalServerError)?;

        match orders.is_empty() {
            true => Err(poem::error::NotFoundError.into()),
            false => Ok(Json(orders)),
        }
    }

    #[oai(path = "/orders", method = "get")]
    async fn get_all(&self, pool: Data<&PgPool>) -> ClientOrderResponse {
        let query =
            sqlx::query_as!(ClientOrderPayload, "SELECT * FROM client_orders");

        let orders =
            query.fetch_all(pool.0).await.map_err(InternalServerError)?;

        if orders.is_empty() {
            return Err(poem::error::NotFoundError.into());
        }

        Ok(Json(orders))
    }

    #[oai(path = "/orders", method = "post")]
    async fn place_order(
        &self,
        pool: Data<&PgPool>,
        order: Json<ClientOrderPayload>,
    ) -> poem::Result<()> {
        let order = order.0;

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
        .execute(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(())
    }

    #[oai(path = "/orders", method = "put")]
    async fn update_order(
        &self,
        pool: Data<&PgPool>,
        order: Json<ClientOrderPayload>,
    ) -> poem::Result<()> {
        let order = order.0;

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
        .execute(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(())
    }

    #[oai(path = "/orders", method = "delete")]
    async fn delete_order(
        &self,
        pool: Data<&PgPool>,
        order: Json<DeleteOrderPayload>,
    ) -> poem::Result<()> {
        let order = order.0;

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
        .execute(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    color_eyre::install()?;
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::connect(db_url.as_str()).await?;

    // NOTE: Run the migrations, only if necessary
    sqlx::migrate!("./migrations").run(&pool).await?;

    let api_service =
        OpenApiService::new(ClientOrderApi, "ClientOrders", "1.0.0")
            .server("http://localhost:3000");

    let ui = api_service.rapidoc(); //NOTE: Best looking out of the box

    let route = Route::new()
        .nest("/", api_service)
        .nest("/orders/ui", ui)
        .data(pool);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(route)
        .await?;

    Ok(())
}
