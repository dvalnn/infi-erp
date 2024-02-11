use core::fmt;

use sqlx::{
    error::BoxDynError, postgres::types::PgMoney, prelude::FromRow, query_as,
    PgPool,
};

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct ClientOrder {
    pub order_number: i64,
    pub client_name_id: String,
    pub work_piece: String, //TODO: Change to enum if possible
    pub quantity: i32,
    pub due_date: i32,
    pub late_pen: PgMoney,
    pub early_pen: PgMoney,
}

impl fmt::Display for ClientOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\
        \tOrder Number:\t{}\n\
        \tClient:\t\t{}\n\
        \tWork Piece:\t{}\n\
        \tQuantity:\t{}\n\
        \tDue Date:\t{}\n\
        \tLate Pen:\t{:?}\n\
        \tEarly Pen:\t{:?}\n\
        ",
            self.order_number,
            self.client_name_id,
            self.work_piece,
            self.quantity,
            self.due_date,
            self.late_pen,
            self.early_pen,
        )
    }
}

pub async fn update_order(
    new: ClientOrder,
    old_number: i64,
    pool: &PgPool,
) -> Result<(), BoxDynError> {
    sqlx::query!(
        "
    UPDATE
        client_orders
    SET
        client_name_id = $1,
        work_piece = $2,
        quantity = $3,
        due_date = $4,
        late_pen = $5,
        early_pen = $6
    WHERE
        order_number = $7
    ",
        new.client_name_id,
        new.work_piece,
        new.quantity,
        new.due_date,
        new.late_pen,
        new.early_pen,
        old_number,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn place_order(
    order: &ClientOrder,
    pool: &PgPool,
) -> Result<(), BoxDynError> {
    sqlx::query!(
        "
    INSERT INTO
        client_orders
    (
        order_number,
        client_name_id,
        work_piece,
        Quantity,
        due_date,
        late_pen,
        early_pen
    )
    VALUES
        ($1, $2, $3, $4, $5, $6, $7)
    ",
        order.order_number,
        order.client_name_id,
        order.work_piece,
        order.quantity,
        order.due_date,
        order.late_pen,
        order.early_pen
    )
    .execute(pool)
    .await?;

    Ok(())
}

//NOTE: return whether the order was placed or not
pub async fn place_unique_order(
    order: &ClientOrder,
    pool: &PgPool,
) -> Result<(), BoxDynError> {
    let query = query_as!(
        ClientOrder,
        "SELECT * FROM client_orders WHERE order_number = $1",
        order.order_number
    );
    let orders = query.fetch_all(pool).await?;
    if orders.is_empty() {
        place_order(order, pool).await?;
    }
    Ok(())
}

pub async fn fetch_all_orders(
    pool: &PgPool,
) -> Result<Vec<ClientOrder>, BoxDynError> {
    let query = query_as!(
        ClientOrder,
        "SELECT * FROM client_orders ORDER BY order_number"
    );
    let orders = query.fetch_all(pool).await?;
    Ok(orders)
}
