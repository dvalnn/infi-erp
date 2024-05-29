use crate::db_api::{Item, Recipe};

use super::item_handler::{self};

#[derive(Debug)]
pub struct ItemBlueprint {
    item: Item,
    process: Vec<item_handler::Step>,
}

impl ItemBlueprint {
    pub async fn insert_to_db(
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

    pub fn set_start(&mut self, starting_date: i64) {
        self.process
            .last_mut()
            .expect("empty process")
            .transf
            .set_date(starting_date as i32);
    }

    pub fn generate(
        item: Item,
        full_recipe: &[Recipe],
    ) -> anyhow::Result<ItemBlueprint> {
        // for each item generate its components and
        // transformations.
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
