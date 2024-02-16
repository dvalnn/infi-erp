/// NOTE: This is implemented independently of the api module so that it can be
/// extracted into its own crate later on.
use askama::Template;
use poem::{
    error::InternalServerError,
    handler,
    web::{Html, Path},
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Penalty {
    cents: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    client_name_id: String,
    order_number: u32,
    work_piece: String,
    quantity: u32,
    due_date: u32,
    late_pen: Penalty,
    early_pen: Penalty,
}

#[derive(Template)]
#[template(path = "index.html")]
struct OrdersTable {
    orders: Vec<Order>,
}

const API_ENDPOINT: &str = "http://localhost:3000/orders";
async fn fetch_orders(endpoint: &str) -> Result<Vec<Order>, reqwest::Error> {
    let response = reqwest::get(endpoint).await?;
    let orders = response.json().await?;
    Ok(orders)
}

#[handler]
pub async fn render_orders(
    client_opt: Option<Path<String>>,
) -> Result<Html<String>> {
    let mut endpoint = API_ENDPOINT.to_string();
    if let Some(client) = client_opt {
        endpoint.push('/');
        endpoint.push_str(&client);
    }

    tracing::info!("making api request to '{endpoint}'");
    let orders = fetch_orders(&endpoint).await.map_err(InternalServerError)?;
    let rendered = OrdersTable { orders }
        .render()
        .map_err(InternalServerError)?;

    Ok(Html(rendered))
}
