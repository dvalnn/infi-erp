#![forbid(unsafe_code)]
#![allow(dead_code, unused_variables)]

mod api;
mod web;

use api::ClientOrderApi;
use tracing::Level;
use web::render_orders;

use poem::{get, listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;
use sqlx::{error::BoxDynError, postgres::PgPool};

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::connect(db_url.as_str()).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let api_service =
        OpenApiService::new(ClientOrderApi, "ClientOrders", "1.0.0")
            .server("http://localhost:3000");

    let ui = api_service.rapidoc(); //NOTE: Best looking out of the box

    let api_route = Route::new()
        .nest("/", api_service)
        .nest("/orders/ui", ui)
        .data(pool);

    tokio::spawn(async {
        Server::new(TcpListener::bind("0.0.0.0:3000"))
            .run(api_route)
            .await
            .unwrap();
    });

    let web_route = Route::new()
        .at("/", get(render_orders))
        .at("/:name", get(render_orders));

    Server::new(TcpListener::bind("0.0.0.0:3030"))
        .run(web_route)
        .await?;

    Ok(())
}
