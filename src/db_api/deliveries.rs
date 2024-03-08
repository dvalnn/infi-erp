use sqlx::postgres::PgQueryResult;
use sqlx::query;
use sqlx::PgConnection;

#[derive(sqlx::Type)]
#[sqlx(type_name = "delivery_status")]
#[sqlx(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Scheduled,
    Delivered,
    Cancelled,
}

pub struct Delivery {
    order_id: i64,
    status: DeliveryStatus,
    day: Option<i32>,
}

impl Delivery {
    pub async fn insert(
        order_id: i64,
        con: &mut PgConnection,
    ) -> sqlx::Result<PgQueryResult> {
        query!("INSERT INTO deliveries (order_id) VALUES ($1)", order_id,)
            .execute(con)
            .await
    }
}
