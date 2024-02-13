#![forbid(unsafe_code)]
#![allow(dead_code, unused_variables)]

mod db;
mod xml;

use color_eyre::eyre::Result;

use poem::{
    error::InternalServerError, listener::TcpListener, web::Data, EndpointExt,
    Route, Server,
};

use poem_openapi::{
    payload::{Json, PlainText},
    Object, OpenApi, OpenApiService,
};

use sqlx::{
    error::BoxDynError,
    migrate,
    postgres::{self, PgPool},
};
use tracing::info;

use crate::xml::parse_xml;

#[derive(Object)]
struct ClientOrder {
    order_number: i64,
    client_name_id: String,
    work_piece: String,
    quantity: i32,
    due_date: i32,
    late_pen: i64,
    early_pen: i64,
}

type ClientOrderResponse = poem::Result<Json<Vec<ClientOrder>>>;

struct ClientOrderApi;

#[OpenApi]
impl ClientOrderApi {
    #[oai(path = "/orders", method = "get")]
    async fn get_all(&self, pool: Data<&PgPool>) -> ClientOrderResponse {
        // TODO : get orders from db

        Ok(Json(vec![ClientOrder {
            order_number: 1,
            client_name_id: "client".to_string(),
            work_piece: "work".to_string(),
            quantity: 1,
            due_date: 1,
            late_pen: 1,
            early_pen: 1,
        }]))
    }

    #[oai(path = "/orders", method = "post")]
    async fn place_order(
        &self,
        pool: Data<&PgPool>,
        order: Json<ClientOrder>,
    ) -> poem::Result<Json<i64>> {
        // TODO : place order in db
        Ok(Json(1))
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let db_url = "postgres://admin:admin@localhost:5432/infi-postgres";
    let pool = PgPool::connect(db_url).await?;

    let api_service =
        OpenApiService::new(ClientOrderApi, "ClientOrders", "1.0.0")
            .server("http://localhost:3000");
    let ui = api_service.rapidoc(); //NOTE: #1
    let route = Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
        .data(pool);

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(route)
        .await?;

    Ok(())
}

async fn temp_stuff() -> Result<(), BoxDynError> {
    let db_url = "postgres://admin:admin@localhost:5432/infi-postgres";
    let pool = PgPool::connect(db_url).await?;
    migrate!("./migrations").run(&pool).await?; // Runs only if needed

    let file = "mock_dataset.xml";
    let orders = parse_xml(file).await?;
    info!("Parsed orders: {:?}", orders.len());

    let mut handles = Vec::with_capacity(orders.len());
    for order in orders {
        let pool = pool.clone();
        let handle = tokio::task::spawn(async move {
            match db::place_unique_order(&order, &pool).await {
                Ok(_) => {}
                Err(err) => println!("Error: {:?}", err),
            }
        });
        handles.push(handle);
    }

    info!("Orders to place: {:?}", handles.len());
    for handle in handles {
        handle.await?;
    }
    info!("All orders placed");

    let orders = db::fetch_all_orders(&pool).await?;
    info!("Orders fetched: {:?}", orders.len());

    Ok(())
}
