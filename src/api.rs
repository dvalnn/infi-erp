use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use sqlx::PgPool;

use crate::queries::{self, ClientOrder, DeleteOrder};

pub async fn get_all(State(pool): State<PgPool>) -> impl IntoResponse {
    match queries::fetch_all_orders(&pool).await {
        Ok(orders) => (StatusCode::OK, Json(orders)),
        Err(e) => {
            tracing::error!("Error {} | fetching all orders", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

pub async fn get_from_client(
    Extension(pool): Extension<PgPool>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match queries::fetch_client_orders(&pool, &name).await {
        Ok(orders) => (StatusCode::OK, Json(orders)),
        Err(e) => {
            tracing::error!(
                "Error {} | fetching orders for client '{}'",
                e,
                name
            );
            (StatusCode::BAD_REQUEST, Json(vec![]))
        }
    }
}

pub async fn new_order(
    Extension(pool): Extension<PgPool>,
    Json(order): Json<ClientOrder>,
) -> impl IntoResponse {
    match queries::place_new_order(&pool, &order).await {
        Ok(res) => (StatusCode::CREATED, Json(res.rows_affected())),
        Err(e) => {
            tracing::error!("Error {} | creating order: {:#?}", e, order);
            (StatusCode::BAD_REQUEST, Json(0))
        }
    }
}

pub async fn update_order(
    Extension(pool): Extension<PgPool>,
    Json(order): Json<ClientOrder>,
) -> impl IntoResponse {
    match queries::update_order(&pool, &order).await {
        Ok(res) => (StatusCode::OK, Json(res.rows_affected())),
        Err(e) => {
            tracing::error!("Error {} | updating order: {:#?}", e, order);
            (StatusCode::BAD_REQUEST, Json(0))
        }
    }
}

pub async fn delete_order(
    Extension(pool): Extension<PgPool>,
    Json(order): Json<DeleteOrder>,
) -> impl IntoResponse {
    match queries::delete_order(&pool, &order).await {
        Ok(res) => (StatusCode::OK, Json(res.rows_affected())),
        Err(e) => {
            tracing::error!("Error {} | deleting order: {:#?}", e, order);
            (StatusCode::BAD_REQUEST, Json(0))
        }
    }
}
