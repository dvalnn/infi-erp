use sqlx::{postgres::PgListener, PgPool};

use crate::db_api::{self};

pub struct Scheduler {
    pool: PgPool,
    listener: PgListener,
}

impl Scheduler {
    pub fn new(pool: PgPool, listener: PgListener) -> Self {
        Self { pool, listener }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let Scheduler { pool, mut listener } = self;

        listener
            .listen(&db_api::NotificationChannel::NewOrder.to_string())
            .await?;

        loop {
            match listener.recv().await {
                Ok(notif) => {
                    let Ok(order_id) = notif.payload().parse::<i64>() else {
                        tracing::error!(
                            "Error while parsing order id from: {}",
                            notif.payload()
                        );
                        continue;
                    };
                    let mut con = pool.acquire().await?;
                    let order =
                        db_api::Order::get_by_id(order_id, &mut con).await?;

                    tracing::debug!("Received new order: {:?}", order);
                }
                Err(e) => {
                    tracing::error!("{e} while receiving notification");
                }
            }
        }
    }
}
