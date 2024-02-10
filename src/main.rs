#![allow(dead_code)]
mod db;

use db::ClientOrder;
use sqlx::{error::BoxDynError, postgres};

use serde_xml_rs::from_str;

#[derive(Debug, serde::Deserialize)]
struct Dataset {
    #[serde(rename = "$value")]
    orders: Vec<db::RawClientOrder>,
}

async fn parse_xml(data: &str) -> Result<Vec<db::ClientOrder>, BoxDynError> {
    let file = std::fs::read_to_string(data)?;
    let dataset: Dataset = from_str(file.as_str())?;
    let orders = dataset
        .orders
        .into_iter()
        .map(|o| o.try_into())
        .collect::<Vec<Result<ClientOrder, BoxDynError>>>();

    orders
        .into_iter()
        .collect::<Result<Vec<ClientOrder>, BoxDynError>>()
}

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let file = "mock_dataset.xml";
    let orders = parse_xml(file).await?;
    println!("Parsed orders: {:?}", orders.len());

    let db_url = "postgres://admin:admin@localhost:5432/infi-postgres";

    //NOTE: connection pool instead of single connection for better performance
    let pool = postgres::PgPool::connect(db_url).await?;

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

    let mut orders = db::fetch_all_orders(&pool).await?;
    orders.sort_unstable_by(|a, b| a.ordernumber.cmp(&b.ordernumber));
    println!("Orders fetched: {:?}", orders.len());

    Ok(())
}
