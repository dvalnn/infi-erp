mod handlers;

use std::sync::RwLock;

use once_cell::sync::Lazy;
use sqlx::{postgres::PgListener, PgPool};

use crate::{
    db_api::{self, Item},
    scheduler::handlers::{blueprint_handler::ItemBlueprint, order_handler},
};

pub static CURRENT_DATE: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(0));
pub const TIME_IN_DAY: i64 = 60; // in the simulation, 1 day is 60 seconds

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

        let full_recipe =
            order_handler::get_full_recipe(order.piece(), pool).await?;

        let order_items: Vec<Item> = order_handler::gen_items(
            order.piece(),
            order.quantity(),
            Some(order.id()),
        )?;

        tracing::debug!("Generated recipe: {:?}", full_recipe);
        tracing::debug!("Generated order items: {:?}", order_items);

        let current_date = *CURRENT_DATE.read().expect("Lock is poisoned");
        let blueprints = order_items
            .iter()
            .filter_map(|item| {
                let mut bp = match ItemBlueprint::generate(
                    (*item).clone(),
                    &full_recipe,
                ) {
                    Ok(bp) => bp,
                    Err(e) => {
                        tracing::error!("{:?}", e);
                        return None;
                    }
                };

                match bp.schedule(order.due_date(), current_date as i32) {
                    Ok(_) => Some(bp),
                    Err(e) => {
                        tracing::error!("{:?}", e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        if blueprints.len() < order.quantity() as usize {
            anyhow::bail!(
                "Cannot fullfill order {:?}, can only schedule {:?}/{:?} parts",
                order.id(),
                blueprints.len(),
                order.quantity()
            );
        }

        let mut tx = pool.begin().await?;
        for mut bp in blueprints {
            bp.insert(&mut tx).await?;
        }

        // order must be delivered on the last day of the schedule
        // when all the items are ready for now, last day is the due date
        order.schedule(order.due_date(), &mut tx).await?;

        tx.commit().await?;

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
