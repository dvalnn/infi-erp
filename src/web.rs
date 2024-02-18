use askama::Template;
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
