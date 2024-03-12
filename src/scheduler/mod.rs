mod handlers;

use sqlx::{postgres::PgListener, PgPool};

use crate::{
    db_api,
    scheduler::handlers::{item_handler, order_handler},
};

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

        //TODO: may be better to extract the transactions from the handlers

        let piece = order.piece();
        let recipe = order_handler::gen_full_recipe(piece, pool);
        let order_items = order_handler::gen_items(order, pool);

        let (recipe, order_items) = tokio::try_join!(recipe, order_items)?;

        tracing::debug!("Generated recipe: {:?}", recipe);
        tracing::debug!("Generated order items: {:?}", order_items);

        // for each item generate its components and
        // transformations. Then schedule everything
        // and update the item status to "Scheduled"
        let mut pair_vec = Vec::new();
        for item in order_items {
            let item_tf_pairs =
                item_handler::gen_transformations(&recipe, item, pool).await?;

            pair_vec.push(item_tf_pairs);
        }

        //TODO: schedule the transformations

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
