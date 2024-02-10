use core::fmt;

use serde::{Deserialize, Serialize};
use sqlx::{
    error::BoxDynError, postgres::types::PgMoney, prelude::FromRow, query_as,
    PgPool,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RawClientOrder {
    pub ordernumber: i64,
    pub clientnameid: String,
    pub workpiece: String, //TODO: Change to enum if possible
    pub quantity: i32,
    pub duedate: i32,
    pub latepen: String,
    pub earlypen: String,
}

impl TryInto<ClientOrder> for RawClientOrder {
    type Error = BoxDynError;
    fn try_into(self) -> Result<ClientOrder, Self::Error> {
        let mut latepen = self.latepen.clone();
        let mut earlypen = self.earlypen.clone();

        if let Some(index) = self.latepen.find('$') {
            latepen.remove(index);
        }
        if let Some(index) = self.earlypen.find('$') {
            earlypen.remove(index);
        }

        if let Some(index) = latepen.find('.') {
            latepen.remove(index);
        }
        if let Some(index) = earlypen.find('.') {
            earlypen.remove(index);
        }

        let latepen_int = latepen.parse::<i64>().expect("should be a number");
        let latepen = PgMoney::from(latepen_int);
        let earlypen_int = earlypen.parse::<i64>().expect("should be a number");
        let earlypen = PgMoney::from(earlypen_int);

        Ok(ClientOrder {
            ordernumber: self.ordernumber,
            clientnameid: self.clientnameid,
            workpiece: self.workpiece,
            quantity: self.quantity,
            duedate: self.duedate,
            latepen,
            earlypen,
        })
    }
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct ClientOrder {
    pub ordernumber: i64,
    pub clientnameid: String,
    pub workpiece: String, //TODO: Change to enum if possible
    pub quantity: i32,
    pub duedate: i32,
    pub latepen: PgMoney,
    pub earlypen: PgMoney,
}

impl fmt::Display for ClientOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\
        \tOrderNumber:\t{}\n\
        \tClient:\t\t{}\n\
        \tWorkPiece:\t{}\n\
        \tQuantity:\t{}\n\
        \tDueDate:\t{}\n\
        \tLatePen:\t{:?}\n\
        \tEarlyPen:\t{:?}\n\
        ",
            self.ordernumber,
            self.clientnameid,
            self.workpiece,
            self.quantity,
            self.duedate,
            self.latepen,
            self.earlypen,
        )
    }
}

pub async fn update_order(
    new: ClientOrder,
    old_id: String,
    pool: &PgPool,
) -> Result<(), BoxDynError> {
    sqlx::query!(
        "
    UPDATE
        client_orders
    SET
        OrderNumber = $1,
        WorkPiece = $2,
        Quantity = $3,
        DueDate = $4,
        LatePen = $5,
        EarlyPen = $6
    WHERE
        ClientNameId = $7
    ",
        new.ordernumber,
        new.workpiece,
        new.quantity,
        new.duedate,
        new.latepen,
        new.earlypen,
        old_id
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
        OrderNumber,
        ClientNameId,
        WorkPiece,
        Quantity,
        DueDate,
        LatePen,
        EarlyPen
    )
    VALUES
        ($1, $2, $3, $4, $5, $6, $7)
    ",
        order.ordernumber,
        order.clientnameid,
        order.workpiece,
        order.quantity,
        order.duedate,
        order.latepen,
        order.earlypen
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
        "SELECT * FROM client_orders WHERE ordernumber = $1",
        order.ordernumber
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
        "SELECT * FROM client_orders ORDER BY ordernumber"
    );
    let orders = query.fetch_all(pool).await?;
    Ok(orders)
}
