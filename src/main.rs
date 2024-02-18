#![forbid(unsafe_code)]
#![allow(dead_code, unused_variables)]

mod api;
mod queries;
mod web;

use axum::extract::Request;
use axum::routing::get;
use axum::ServiceExt;
use sqlx::{error::BoxDynError, postgres::PgPool};
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

async fn connect_to_db() -> Result<PgPool, BoxDynError> {
    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::connect(db_url.as_str()).await?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "example_tracing_aka_logging=debug,tower_http=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = connect_to_db().await?;

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/api/orders",
            get(api::get_all)
                .post(api::new_order)
                .put(api::update_order)
                .delete(api::delete_order),
        )
        .route("/api/orders/:name", get(api::get_from_client))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let app = NormalizePathLayer::trim_trailing_slash().layer(app);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::debug!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await?;

    Ok(())
}
