#![allow(dead_code)]

use core::fmt;

use sqlx::{
    error::BoxDynError,
    postgres::{self, types::PgMoney},
    prelude::FromRow,
    query_as, PgPool,
};

#[derive(Debug, FromRow)]
struct ClientOrder {
    clientnameid: String,
    ordernumber: i32,
    workpiece: String, //TODO: Change to enum later
    quantity: i32,
    duedate: i32,
    latepen: PgMoney,
    earlypen: PgMoney,
}

impl fmt::Display for ClientOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\
            \tClient:\t\t{}\n\
            \tOrderNumber:\t{}\n\
            \tWorkPiece:\t{}\n\
            \tQuantity:\t{}\n\
            \tDueDate:\t{}\n\
            \tLatePen:\t{:?}\n\
            \tEarlyPen:\t{:?}\n\
            ",
            self.clientnameid,
            self.ordernumber,
            self.workpiece,
            self.quantity,
            self.duedate,
            self.latepen,
            self.earlypen,
        )
    }
}

async fn update_order(
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

async fn place_order(
    order: ClientOrder,
    pool: &PgPool,
) -> Result<(), BoxDynError> {
    sqlx::query!(
        "
        INSERT INTO
            client_orders
        (
            ClientNameId,
            OrderNumber,
            WorkPiece,
            Quantity,
            DueDate,
            LatePen,
            EarlyPen
        )
        VALUES
            ($1, $2, $3, $4, $5, $6, $7)
        ",
        order.clientnameid,
        order.ordernumber,
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

async fn fetch_all_orders(
    pool: &PgPool,
) -> Result<Vec<ClientOrder>, BoxDynError> {
    let query = query_as!(
        ClientOrder,
        "
        SELECT
            clientnameid,
            ordernumber,
            workpiece,
            quantity,
            duedate,
            latepen,
            earlypen
        FROM
            client_orders
        "
    );
    let orders = query.fetch_all(pool).await?;
    Ok(orders)
}

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let url = "postgres://admin:admin@localhost:5432/infi-postgres";
    //NOTE: connection pool instead of single connection for better performance
    let pool = postgres::PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let order = ClientOrder {
        clientnameid: "lucas".to_string(),
        ordernumber: 29,
        workpiece: "P1".to_string(),
        quantity: 3,
        duedate: 16,
        latepen: PgMoney(1500), // TODO: fix this repr in the database
        earlypen: PgMoney(1000),
    };

    update_order(order, "lucas".to_string(), &pool).await?;

    let orders = fetch_all_orders(&pool).await?;

    for order in orders {
        println!("order: {}", order);
    }

    Ok(())
}
