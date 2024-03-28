use crate::{
    db_api::{Item, Recipe},
    scheduler::TIME_IN_DAY,
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

    pub fn schedule(
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
            if duration_acc > TIME_IN_DAY {
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

    pub fn generate(
        item: Item,
        full_recipe: &[Recipe],
    ) -> anyhow::Result<ItemBlueprint> {
        // for each item generate its components and
        // transformations. Then schedule everything
        // and update the item status to "Scheduled"
        let process =
            match item_handler::describe_process(full_recipe, item.clone()) {
                Ok(proc) => proc,
                Err(e) => anyhow::bail!("{:?}", e),
            };

        Ok(Self { item, process })
    }

    pub fn item(&self) -> &Item {
        &self.item
    }

    pub fn process_mut(&mut self) -> &mut Vec<item_handler::Step> {
        &mut self.process
    }
}
