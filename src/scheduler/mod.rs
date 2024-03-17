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

impl ItemBlueprint {
    fn schedule(&mut self, due_date: i32) -> anyhow::Result<i32> {
        //TODO: Get current day, for now assume 0

        let mut schedule_day = due_date;
        let mut duration_acc = 0;

        // walk back from the due date and find the
        // last day that can accommodate each step in
        // the process
        let mut dates = Vec::new();

        for step in &self.process {
            duration_acc += step.recipe.operation_time;
            if duration_acc > Scheduler::TIME_IN_DAY {
                schedule_day -= 1;
                duration_acc = 0;
            }

            // TODO: check against the current day
            if schedule_day < 0 {
                return Err(anyhow::anyhow!(
                    "Cannot schedule item, due date is in the past"
                ));
            }

            dates.push(schedule_day);
        }

        self.process.iter_mut().zip(dates).for_each(|(step, date)| {
            step.transf.set_date(date);
            tracing::debug!(
                "Scheduled transformation {:?} for day {}",
                step.transf,
                date
            );
        });

        Ok(schedule_day)
    }
}

pub struct Scheduler {
    pool: PgPool,
    listener: PgListener,
}

impl Scheduler {
    pub const TIME_IN_DAY: i64 = 60; // in the simulation, 1 day is 60 seconds

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

        let recipe =
            order_handler::get_full_recipe(order.piece(), pool).await?;

        let order_items: Vec<Item> = order_handler::gen_items(
            order.piece(),
            order.quantity(),
            Some(order.id()),
        )?;

        tracing::debug!("Generated recipe: {:?}", recipe);
        tracing::debug!("Generated order items: {:?}", order_items);

        //TODO: this block can be cleaned up, extracted and parallelized
        let blueprints = {
            // for each item generate its components and
            // transformations. Then schedule everything
            // and update the item status to "Scheduled"
            let mut blueprints =
                order_items.into_iter().fold(Vec::new(), |mut acc, item| {
                    let process = match item_handler::describe_process(
                        &recipe,
                        item.clone(),
                    ) {
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

            //TODO: query MES to get avg work efficiency for this item
            //      for now assume minimum efficiency which means, only
            //      1 item can be processed at a time
            let mut due_date = order.due_date();
            blueprints.iter_mut().for_each(|bp| {
                match bp.schedule(due_date) {
                    Ok(day) => {
                        tracing::info!(
                            "Scheduled process with {:?} steps for item {:?}",
                            bp.process.len(),
                            bp.item
                        );
                        due_date = day;
                    }
                    Err(e) => tracing::error!("{:?}", e),
                };
            });

            let scheduled = blueprints.len() as i32;
            if scheduled < order.quantity() {
                anyhow::bail!(
                    "Order {:?} cannot be fullfilled: \
                {:?}/{:?} items cannot be scheduled",
                    order.id(),
                    order.quantity() - scheduled,
                    order.quantity()
                )
            }

            blueprints
        };

        let mut tx = pool.begin().await?;

        for bp in blueprints {
            bp.item.insert(&mut tx).await?;
            for step in bp.process {
                step.material.insert(&mut tx).await?;
                step.transf.insert(&mut tx).await?;
            }
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
