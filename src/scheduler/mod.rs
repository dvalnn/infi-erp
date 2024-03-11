mod handlers;

use sqlx::{postgres::PgListener, PgPool};

use crate::{db_api, scheduler::handlers::order_handler};

pub struct Scheduler {
    pool: PgPool,
    listener: PgListener,
}

impl Scheduler {
    pub fn new(pool: PgPool, listener: PgListener) -> Self {
        Self { pool, listener }
    }

    pub async fn process_notif(
        payload: &str,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        let order_id = payload.parse::<i64>()?;

        let order = {
            let mut con = pool.acquire().await?;
            db_api::Order::get_by_id(order_id, &mut con).await?
        };

        tracing::debug!("Received new order: {:?}", order);

        let piece = order.piece();
        let recipe = order_handler::gen_full_recipe(piece, pool);
        let order_items = order_handler::gen_items(order, pool);

        let (recipe, order_items) = tokio::try_join!(recipe, order_items)?;

        tracing::debug!("Generated recipe: {:?}", recipe);
        tracing::debug!("Generated order items: {:?}", order_items);

        // for each item generate its components and
        // transformations. Then schedule everything
        // and update the item status to "Scheduled"

        Ok(())
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        self.listener
            .listen(&db_api::NotificationChannel::NewOrder.to_string())
            .await?;

        loop {
            let notif = match self.listener.recv().await {
                Ok(notif) => notif,
                Err(e) => {
                    tracing::error!("{:?}", e);
                    continue;
                }
            };

            match Self::process_notif(notif.payload(), &self.pool).await {
                Ok(_) => (),
                Err(e) => tracing::error!("{:?}", e),
            }
        }
    }
}
