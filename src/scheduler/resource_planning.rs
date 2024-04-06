use std::collections::{BTreeMap, HashMap};

use sqlx::PgPool;

use crate::db_api::{
    RawMaterial, Shippment, Supplier, UnderAllocatedShippment,
};

use super::Scheduler;

// TODO: Take warehouse capacity into account

#[allow(dead_code, unused_mut, unused_variables)]
pub async fn process_material_variant(
    variant: RawMaterial,
    pool: &PgPool,
) -> anyhow::Result<()> {
    tracing::info!("Processing {:?} needs", variant);

    let current_day = Scheduler::get_date() as i32;

    // 1. Get net requirements for the variant by day,
    //    Get under alocated incomming shippments
    //    Get available suppliers
    let (mut net_req, suppliers, mut under_allocd_shippment_map) =
        query_needed_data(pool, variant).await?;

    tracing::trace!("Net requirements: {:?}", net_req);
    tracing::trace!("Suppliers: {:?}", suppliers);
    tracing::trace!(
        "Under allocated shippments: {:?}",
        under_allocd_shippment_map
    );

    // 2. Remove from net requirements the stock already ordered in the past
    for (day, quantity) in net_req.iter_mut() {
        let Some(under_allocated) = under_allocd_shippment_map.get_mut(day)
        else {
            continue;
        };

        for s in under_allocated.iter_mut() {
            if *quantity == 0 {
                break;
            }

            let allocated = s.extra_quantity.min(*quantity as i64);
            *quantity -= allocated as i32;
            s.extra_quantity -= allocated;
            s.added = Some(allocated);

            tracing::info!(
                "Allocated {} from shippment {} for day {:#?} {} needs",
                allocated,
                s.id,
                variant,
                day
            );
        }

        // remove shippments to which nothing was allocated
        under_allocated.retain(|s| s.added.is_some());
    }

    //3. retain only days with net requirements
    //   retain only under allocated shippments to which we allocated stock
    net_req.retain(|_, quantity| *quantity > 0);
    under_allocd_shippment_map.retain(|_, shippments| !shippments.is_empty());
    tracing::trace!("Net requirements after allocation: {:?}", net_req);

    // 3. Create a purchase order for each supplier for each day
    // Fill low demand days with extra to reach the minimum order quantity.
    let mut purchase_orders = HashMap::new();
    for (due_date, quantity) in net_req.iter() {
        let day = *due_date;
        let available_time = day - current_day;

        if available_time < 0 {
            tracing::warn!("Material for day {} is already due", day);
            continue;
        }

        let suppliers = suppliers
            .iter()
            .filter(|s| s.can_deliver_in(available_time));

        let Some(cheapest_purchase) = suppliers
            .into_iter()
            .map(|s| s.shippment_details(*quantity))
            .min_by_key(|order| order.cost().0)
        else {
            tracing::warn!(
                "No supplier can deliver {:#?} in time for day {}",
                variant,
                day
            );
            continue;
        };

        purchase_orders.insert(day, cheapest_purchase);
    }

    Ok(())
}

async fn query_needed_data(
    pool: &PgPool,
    variant: RawMaterial,
) -> anyhow::Result<(
    BTreeMap<i32, i32>,
    Vec<Supplier>,
    HashMap<i32, Vec<UnderAllocatedShippment>>,
)> {
    let mut conn = pool.acquire().await.unwrap();

    let net_req = variant.get_net_requirements(&mut conn).await?;
    let suppliers = Supplier::get_by_item_kind(variant, &mut conn).await?;

    let mut shippment_map = HashMap::new();
    for day in net_req.keys() {
        let shippments =
            Shippment::get_under_allocated(*day, variant, &mut conn).await?;
        if !shippments.is_empty() {
            shippment_map.insert(*day, shippments);
        }
    }

    Ok((net_req, suppliers, shippment_map))
}
