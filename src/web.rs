use askama::Template;
use axum::extract::{Path, State};
use sqlx::PgPool;

use crate::queries::{self, OrderDetails};

#[derive(Template, Debug)]
#[template(path = "index.html")]
pub struct OrdersTable {
    orders: Vec<OrderDetails>,
}

pub async fn orders(State(pool): State<PgPool>) -> OrdersTable {
    let orders = queries::fetch_all_orders(&pool).await;
    match orders {
        Ok(orders) => OrdersTable { orders },
        Err(_) => OrdersTable { orders: Vec::new() },
    }
}

pub async fn orders_from(
    State(pool): State<PgPool>,
    Path(name): Path<String>,
) -> OrdersTable {
    let orders = queries::fetch_client_orders(&pool, &name).await;
    match orders {
        Ok(orders) => OrdersTable { orders },
        Err(_) => OrdersTable { orders: Vec::new() },
    }
}
