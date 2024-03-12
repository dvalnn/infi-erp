mod handlers;

use sqlx::{postgres::PgListener, PgPool};

use crate::{
    db_api::{self, Item},
    scheduler::handlers::{item_handler, order_handler},
};

#[derive(Debug)]
struct ItemBlueprint {
    item: Item,
    process: Vec<item_handler::Step>,
}

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
        let order_id = uuid::Uuid::parse_str(payload)?;

        let order = {
            let mut con = pool.acquire().await?;
            db_api::Order::get_by_id(order_id, &mut con).await?
        };

        tracing::debug!("Received new order: {:?}", order);

        //TODO: may be better to extract the transactions from the handlers

        let recipe =
            order_handler::get_full_recipe(order.piece(), pool).await?;

        let order_items = order_handler::gen_items(
            order.piece(),
            order.quantity(),
            Some(order.id()),
        )?;

        tracing::debug!("Generated recipe: {:?}", recipe);
        tracing::debug!("Generated order items: {:?}", order_items);

        // for each item generate its components and
        // transformations. Then schedule everything
        // and update the item status to "Scheduled"
        let blueprints =
            order_items.into_iter().fold(Vec::new(), |mut acc, item| {
                let process =
                    match item_handler::describe_process(&recipe, item.clone())
                    {
                        Ok(proc) => proc,
                        Err(e) => {
                            tracing::error!("{:?}", e);
                            return acc;
                        }
                    };

                acc.push(ItemBlueprint {
                    item: item.clone(),
                    process,
                });
                acc
            });

        tracing::trace!("Generated blueprints: {:?}", blueprints);

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
