use std::collections::HashMap;

use sqlx::PgPool;

use crate::db_api::{
    MaterialShipment, RawMaterial, Shipment, Supplier, UnderAllocatedShipment,
};

use super::Scheduler;

// TODO: Take warehouse capacity into account
pub async fn resolve_material_needs(
    variant: RawMaterial,
    pool: PgPool,
) -> anyhow::Result<()> {
    tracing::info!("Processing {:?} needs", variant);

    // 1. Get net requirements for the variant by day,
    //    Get under alocated incomming shipments
    //    Get available suppliers
    let qr = query_needed_data(&pool, variant).await?;
    if qr.net_req.is_empty() {
        tracing::info!("No {:#?} needs at the moment", variant);
        return Ok(());
    }

    tracing::info!(
        "Net {:#?} requirements ({{day: ammount}}): {:?}",
        variant,
        qr.net_req
    );
    tracing::trace!("{:#?} suppliers: {:?}", variant, qr.suppliers);
    tracing::trace!("Under allocated shipments: {:?}", qr.shipments);

    // 2. Process the data to create purchase orders
    let pr = process_purchases(qr);

    tracing::debug!(
        "Altered shipments: {:#?}",
        pr.altered_shipments_by_due_date
    );
    tracing::debug!(
        "New Purchase orders: {:#?}",
        pr.purchase_orders_by_due_date
    );

    // 3. Get pending items from database
    let mut tx = pool.begin().await?;
    let mut pending = variant.get_pending_purchase(&mut tx).await?;
    let mut material_shipments = Vec::<MaterialShipment>::new();

    //4. Link pending items to the altered existing shipments
    for (due_date, shipments) in pr.altered_shipments_by_due_date {
        for s in shipments {
            let items_to_insert = pending
                .iter()
                .filter(|p| p.due_date == due_date)
                .take(s.added as usize)
                .map(|p| MaterialShipment::new(p.item_id, s.id))
                .collect::<Vec<_>>();

            pending.retain(|p| {
                !items_to_insert
                    .iter()
                    .any(|i| i.raw_material_id() == p.item_id)
            });

            material_shipments.extend(items_to_insert);
        }
    }

    // 5. Insert new shipments into de dabase to get their IDs
    // 6. Link remaining pending items to new purchase orders
    for (due_date, po) in pr.purchase_orders_by_due_date {
        let id = po.insert(&mut tx).await?;

        let items_to_insert = pending
            .iter()
            .filter(|p| p.due_date == due_date)
            .take(po.quantity() as usize)
            .map(|p| MaterialShipment::new(p.item_id, id))
            .collect::<Vec<_>>();

        pending.retain(|p| {
            !items_to_insert
                .iter()
                .any(|i| i.raw_material_id() == p.item_id)
        });
        material_shipments.extend(items_to_insert);
    }

    // 7. Insert the new populate the material shippemnts join table
    //    with the new tuples
    for ms in material_shipments {
        ms.insert(&mut tx).await?;
    }

    tx.commit().await?;

    tracing::info!("Resolved {:#?} needs", variant);

    Ok(())
}

struct PurchaseProcessingResults {
    pub purchase_orders_by_due_date: HashMap<i32, Shipment>,
    pub altered_shipments_by_due_date: HashMap<i32, Vec<AlteredShippment>>,
}

#[derive(Debug)]
struct AlteredShippment {
    pub id: i64,
    pub added: i64,
}

fn process_purchases(mut qr: QueryResults) -> PurchaseProcessingResults {
    // 1. Remove from net requirements the stock already ordered in the past
    process_under_alocated_shipments(
        &mut qr.net_req,
        &mut qr.shipments,
        qr.material_kind,
    );

    // 2. retain only days with net requirements
    //    retain only under allocated shipments to which stock
    //    was allocated and need to be updated onnthe database
    qr.net_req.retain(|_, quantity| *quantity > 0);
    qr.shipments.retain(|_, shipments| !shipments.is_empty());
    tracing::trace!(
        "Net requirements after shippment adjusts: {:?}",
        qr.net_req
    );

    let altered_shippments = qr
        .shipments
        .iter()
        .map(|(day, shipments)| {
            let altered = shipments
                .iter()
                .map(|s| AlteredShippment {
                    id: s.id,
                    added: s.added.expect("added is always Some"),
                })
                .collect();
            (*day, altered)
        })
        .collect();

    // 3. Create a purchase order for each supplier for each day
    // Fill low demand days with extra to reach the minimum order quantity.
    let mut purchase_orders = HashMap::new();
    for (due_date, quantity) in qr.net_req.iter() {
        let day = *due_date;
        let available_time = day - qr.current_date as i32;

        if available_time < 0 {
            tracing::warn!("Material for day {} is already due", day);
            continue;
        }

        let suppliers = qr.suppliers.clone();
        let Some(cheapest_purchase) = suppliers
            .into_iter()
            .filter_map(|s| match s.can_deliver_in(available_time) {
                true => Some(s.shippment(*quantity, *due_date)),
                false => None,
            })
            .min_by_key(|shippment| shippment.cost().0)
        else {
            tracing::warn!(
                "No supplier can deliver {:#?} in time for day {}",
                qr.material_kind,
                day
            );
            continue;
        };

        purchase_orders.insert(day, cheapest_purchase);
    }

    PurchaseProcessingResults {
        purchase_orders_by_due_date: purchase_orders,
        altered_shipments_by_due_date: altered_shippments,
    }
}

fn process_under_alocated_shipments(
    net_req: &mut HashMap<i32, i32>,
    shipments: &mut HashMap<i32, Vec<UnderAllocatedShipment>>,
    material_kind: RawMaterial,
) {
    for (day, quantity) in net_req.iter_mut() {
        let Some(under_allocated) = shipments.get_mut(day) else {
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
                "Allocated {} free slot from shippment id {} for day {}'s {:#?} needs",
                allocated,
                s.id,
                day,
                material_kind
            );
        }

        // remove shipments to which nothing was allocated
        under_allocated.retain(|s| s.added.is_some());
    }
}

struct QueryResults {
    pub current_date: u32,
    pub shipments: HashMap<i32, Vec<UnderAllocatedShipment>>,
    pub suppliers: Vec<Supplier>,
    pub net_req: HashMap<i32, i32>,
    pub material_kind: RawMaterial,
}

async fn query_needed_data(
    pool: &PgPool,
    variant: RawMaterial,
) -> anyhow::Result<QueryResults> {
    let mut conn = pool.acquire().await.unwrap();

    let net_req = variant.get_net_requirements(&mut conn).await?;
    let suppliers = Supplier::get_by_item_kind(variant, &mut conn).await?;

    let mut shipment_map = HashMap::new();
    for day in net_req.keys() {
        let shipments =
            Shipment::get_under_allocated(*day, variant, &mut conn).await?;
        if !shipments.is_empty() {
            shipment_map.insert(*day, shipments);
        }
    }

    let query_results = QueryResults {
        current_date: Scheduler::get_date(),
        shipments: shipment_map,
        suppliers,
        net_req,
        material_kind: variant,
    };

    Ok(query_results)
}
