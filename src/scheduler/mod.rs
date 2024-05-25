mod handlers;
mod resource_planning;

use sqlx::{postgres::PgListener, PgPool};

use crate::{
    db_api::{self, Item, NotificationChannel as NotifCh, RawMaterial},
    scheduler::handlers::{blueprint_handler::ItemBlueprint, order_handler},
};

pub const TIME_IN_DAY: i64 = 60; // in the simulation, 1 day is 60 seconds

pub struct Scheduler {
    pool: PgPool,
    listener: PgListener,
}

impl Scheduler {
    pub fn new(pool: PgPool, listener: PgListener) -> Self {
        Self { pool, listener }
    }

    async fn process_new_order(
        payload: impl ToString,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        let order_id = uuid::Uuid::parse_str(&payload.to_string())?;

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

        let current_date = {
            let mut con = pool.acquire().await?;
            db_api::get_date(&mut con).await?
        };

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
            bp.insert_to_db(&mut tx).await?;
        }

        // order must be delivered on the last day of the schedule
        // when all the items are ready for now, last day is the due date
        order.schedule(order.due_date(), &mut tx).await?;

        tx.commit().await?;

        let mut con = pool.acquire().await?;
        NotifCh::notify(
            NotifCh::MaterialsNeeded,
            &order.id().to_string(),
            &mut con,
        )
        .await?;

        Ok(())
    }

    async fn process_material_needs(
        _: impl ToString,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        let raw_material_variants =
            enum_iterator::all::<RawMaterial>().collect::<Vec<_>>();

        let mut set = tokio::task::JoinSet::new();

        for variant in raw_material_variants {
            set.spawn(resource_planning::resolve_material_needs(
                variant,
                pool.clone(),
            ));
        }

        while let Some(join_res) = set.join_next().await {
            match join_res {
                Ok(task_res) => {
                    if let Err(e) = task_res {
                        tracing::error!("{:?}", e)
                    }
                }
                Err(e) => anyhow::bail!("{:?}", e),
            }
        }

        Ok(())
    }

    pub async fn process_notif(
        notif: sqlx::postgres::PgNotification,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        match NotifCh::try_from(notif.channel())? {
            NotifCh::NewOrder => {
                Self::process_new_order(notif.payload(), pool).await
            }
            NotifCh::MaterialsNeeded => {
                tracing::info!(
                    "Materials needed for order: {:?}",
                    notif.payload()
                );
                Self::process_material_needs(notif.payload(), pool).await
            }
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        self.listener.listen(&NotifCh::NewOrder.to_string()).await?;
        self.listener
            .listen(&NotifCh::MaterialsNeeded.to_string())
            .await?;

        loop {
            let notif = match self.listener.recv().await {
                Ok(notif) => notif,
                Err(e) => {
                    tracing::error!("{:?}", e);
                    continue;
                }
            };

            match Self::process_notif(notif, &self.pool).await {
                Ok(_) => (),
                Err(e) => tracing::error!("{:?}", e),
            }
        }
    }
}
