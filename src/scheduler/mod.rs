mod handlers;
mod resource_planning;

use std::{collections::HashMap, sync::RwLock};

use once_cell::sync::Lazy;
use sqlx::{postgres::PgListener, PgPool};

use crate::{
    db_api::{
        self, Item, MaterialShippments, NotificationChannel as NotifCh,
        RawMaterial, RawMaterialDetails, Shippment, Supplier,
    },
    scheduler::handlers::{blueprint_handler::ItemBlueprint, order_handler},
};

pub const TIME_IN_DAY: i64 = 60; // in the simulation, 1 day is 60 seconds
pub static CURRENT_DATE: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(1));

pub struct Scheduler {
    pool: PgPool,
    listener: PgListener,
}

impl Scheduler {
    pub fn new(pool: PgPool, listener: PgListener) -> Self {
        Self { pool, listener }
    }

    pub fn get_date() -> u32 {
        *CURRENT_DATE.read().expect("lock was poisoned")
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

        let current_date = Scheduler::get_date();
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

    async fn process_material_variant(
        variant: RawMaterial,
        pool: PgPool,
    ) -> anyhow::Result<()> {
        tracing::info!("Processing material variant: {:?}", variant);

        let current_date = Self::get_date();

        let mut con = pool.acquire().await?;
        let pending_by_day = variant.get_pending(&mut con).await?.iter().fold(
            HashMap::<i32, Vec<RawMaterialDetails>>::new(),
            |mut map, material| {
                map.entry(material.due_date)
                    .or_default()
                    .push(material.clone());
                map
            },
        );

        if pending_by_day.is_empty() {
            anyhow::bail!("Order has no raw material requirements")
        }

        tracing::info!(
            "[Variant {:#?}] {} days with pending materials",
            variant,
            pending_by_day.len()
        );

        let earliest_due = *pending_by_day.keys().min().expect("min exists");
        if earliest_due <= current_date as i32 {
            //TODO: Send cancel order signal
            anyhow::bail!("unfulfillable raw_material request")
        }

        let mut shippment_day_map = HashMap::<i32, Option<Shippment>>::new();

        for day in pending_by_day.keys() {
            let shippments = Shippment::get_existing_shippment(
                variant,
                *day,
                current_date as i32,
                &mut con,
            )
            .await?;

            shippment_day_map.insert(*day, shippments);
        }

        let some_shippement_days = shippment_day_map
            .iter()
            .filter(|(_, shippements)| shippements.is_some())
            .map(|(day, _)| *day)
            .collect::<Vec<_>>();

        tracing::info!(
            "[Variant {:#?}] {} days with existing shippments",
            variant,
            some_shippement_days.len()
        );

        for shippement_day in some_shippement_days {
            let shippment = shippment_day_map
                .get_mut(&shippement_day)
                .expect("day exists")
                .as_mut()
                .expect("shippment exists");

            shippment.add_to_quantity(
                pending_by_day
                    .get(&shippement_day)
                    .expect("day exists")
                    .len() as i32,
            );
        }

        let none_shippement_days = shippment_day_map
            .iter()
            .filter(|(_, shippements)| shippements.is_none())
            .map(|(day, _)| *day)
            .collect::<Vec<_>>();

        for arrival_day in none_shippement_days {
            let time = arrival_day - current_date as i32;
            let suppliers =
                Supplier::get_compatible(variant, time, &mut con).await?;

            if suppliers.is_empty() {
                //TODO: send order reschedule signal
                anyhow::bail!("No supplier can deliver in time")
            }
            tracing::info!(
                "[Variant {:#?}] {} suppliers can deliver on day {}",
                variant,
                suppliers.len(),
                arrival_day
            );

            let cheapest_supplier = suppliers
                .iter()
                .min_by_key(|s| s.unit_price().0)
                .expect("supplier exists");

            let request_deadline =
                arrival_day - cheapest_supplier.delivery_time();

            let order_quantity =
                pending_by_day.get(&arrival_day).expect("day exists").len()
                    as i64;

            let order_cost = order_quantity * cheapest_supplier.unit_price().0;

            let shippment = Shippment::new(
                cheapest_supplier.id(),
                request_deadline,
                order_quantity as i32,
                order_cost.into(),
            );

            shippment_day_map.insert(arrival_day, Some(shippment));
        }

        let mut tx = pool.begin().await?;
        for (day, shippment) in shippment_day_map {
            let shippment = shippment.expect("shippment exists");
            let shippment_id = shippment.upsert(&mut tx).await?;
            tracing::info!(
                "[Variant {:#?}] Created shippment with id: {}",
                variant,
                shippment_id
            );
            let items = pending_by_day.get(&day).expect("day exists");
            for item in items {
                MaterialShippments::new(item.item_id, shippment_id)
                    .insert(&mut tx)
                    .await?;
            }
        }

        tx.commit().await?;

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
            set.spawn(Self::process_material_variant(variant, pool.clone()));
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
