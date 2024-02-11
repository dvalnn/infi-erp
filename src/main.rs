#![allow(dead_code)]
mod db;
mod xml;

use sqlx::{error::BoxDynError, migrate, postgres};

use crate::xml::parse_xml;

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let db_url = "postgres://admin:admin@localhost:5432/infi-postgres";
    //NOTE: connection pool instead of single connection for better performance
    let pool = postgres::PgPool::connect(db_url).await?;
    migrate!("./migrations").run(&pool).await?; // Runs only if needed

    let file = "mock_dataset.xml";
    let orders = parse_xml(file).await?;
    println!("Parsed orders: {:?}", orders.len());

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

    println!("Orders to place: {:?}", handles.len());
    for handle in handles {
        handle.await?;
    }
    println!("All orders placed");

    let orders = db::fetch_all_orders(&pool).await?;
    println!("Orders fetched: {:?}", orders.len());

    Ok(())
}
