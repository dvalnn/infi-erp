#![allow(dead_code)]

use sqlx::{
    error::BoxDynError,
    postgres::{self, types::PgMoney},
};

mod db;

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let db_url = "postgres://admin:admin@localhost:5432/infi-postgres";
    //NOTE: connection pool instead of single connection for better performance
    let pool = postgres::PgPool::connect(db_url).await?;

    let template_order = db::ClientOrder {
        ordernumber: 2, // NOTE: probably a unique identifier
        clientnameid: "lucas".to_string(),
        workpiece: "P9".to_string(),
        quantity: 20,
        duedate: 11,
        latepen: PgMoney(1000),
        earlypen: PgMoney(9000),
    };

    let mut orders = db::fetch_all_orders(&pool).await?;
    if orders.is_empty()
        || !orders
            .iter()
            .any(|o| o.ordernumber == template_order.ordernumber)
    {
        db::place_order(template_order, &pool).await?;
        orders = db::fetch_all_orders(&pool).await?;
    }

    for order in orders {
        println!("order: {}", order);
    }

    Ok(())
}
