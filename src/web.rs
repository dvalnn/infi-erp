/// NOTE: This is implemented independently of the api module so that it can be
///       extracted into its own crate later on.
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

#[derive(Template, Debug)]
#[template(path = "index.html")]
struct OrdersTable {
    orders: Vec<Order>,
}

const API_ENDPOINT: &str = "http://localhost:3000/orders";

//TODO: Display an empty table when no orders are found
#[handler]
pub async fn render_orders(
    client_opt: Option<Path<String>>,
) -> Result<Html<String>> {
    let mut endpoint = API_ENDPOINT.to_string();
    if let Some(client) = client_opt {
        endpoint.push('/');
        endpoint.push_str(&client);
    }

    let response =
        reqwest::get(&endpoint).await.map_err(InternalServerError)?;

    if reqwest::StatusCode::NOT_FOUND == response.status() {
        tracing::debug!("No orders found for: {}", endpoint);
        return Err(poem::http::StatusCode::NO_CONTENT.into());
    }

    let orders = response.json().await.map_err(InternalServerError)?;
    tracing::debug!("{:#?}", orders);

    let rendered = OrdersTable { orders }.render();
    match rendered {
        Ok(rendered) => Ok(Html(rendered)),
        Err(e) => {
            tracing::error!("Error rendering orders: {}", e);
            Err(InternalServerError(e))
        }
    }
}
