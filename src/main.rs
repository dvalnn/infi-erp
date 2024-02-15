#![forbid(unsafe_code)]

mod api;

use api::ClientOrderApi;
use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;
use sqlx::{error::BoxDynError, postgres::PgPool};

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::connect(db_url.as_str()).await?;

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
