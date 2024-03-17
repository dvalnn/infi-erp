use crate::{
    db_api::{Item, Order, Recipe},
    scheduler::Scheduler,
};

use super::item_handler::{self};

#[derive(Debug)]
pub struct ItemBlueprint {
    item: Item,
    process: Vec<item_handler::Step>,
}

impl ItemBlueprint {
    pub async fn insert(
        &mut self,
        con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<()> {
        self.item().insert(con).await?;
        for step in self.process_mut() {
            step.material.insert(con).await?;
            step.transf.insert(con).await?;
        }

        Ok(())
    }

    fn schedule(
        &mut self,
        due_date: i32,
        current_date: i32,
    ) -> anyhow::Result<i32> {
        let mut schedule_day = due_date;
        let mut duration_acc = 0;

        // walk back from the due date and find the
        // last day that can accommodate each step in
        // the process
        for step in self.process.iter_mut() {
            duration_acc += step.recipe.operation_time;
            if duration_acc > Scheduler::TIME_IN_DAY {
                schedule_day -= 1;
                duration_acc = 0;
            }

            if schedule_day < current_date {
                return Err(anyhow::anyhow!(
                    "Cannot schedule item, due date is in the past"
                ));
            }

            step.transf.set_date(schedule_day);

            tracing::debug!(
                "Scheduled transformation {:?} for day {}",
                step.transf,
                schedule_day
            );
        }

        Ok(schedule_day)
    }

    pub fn generate_scheduled(
        order: &Order,
        order_items: Vec<Item>,
        full_recipe: Vec<Recipe>,
    ) -> anyhow::Result<Vec<ItemBlueprint>> {
        // for each item generate its components and
        // transformations. Then schedule everything
        // and update the item status to "Scheduled"
        let mut blueprints =
            order_items.into_iter().fold(Vec::new(), |mut acc, item| {
                let process = match item_handler::describe_process(
                    &full_recipe,
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
            //TODO: query MES to get current date
            match bp.schedule(due_date, 0) {
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

        Ok(blueprints)
    }

    pub fn item(&self) -> &Item {
        &self.item
    }

    pub fn process_mut(&mut self) -> &mut Vec<item_handler::Step> {
        &mut self.process
    }
}
