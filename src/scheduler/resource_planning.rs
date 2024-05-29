use sqlx::PgPool;

use crate::db_api::{
    MaterialShipment, RawMaterial, Shipment, Supplier, UnderAllocatedShipment,
};

struct DayVariantNeedsData {
    pub due_date: i32,
    pub net_req: i32,
    pub variant: RawMaterial,
    pub under_allocated: Vec<UnderAllocatedShipment>,
    pub suppliers: Vec<Supplier>,
}

async fn resolve_day_needs(
    pool: &PgPool,
    mut needs: DayVariantNeedsData,
) -> anyhow::Result<()> {
    // 2. Process the data to create purchase orders
    let current_date = {
        let mut conn = pool.acquire().await?;
        crate::db_api::get_date(&mut conn).await?
    };

    let pr = process_purchases(&mut needs, current_date);

    tracing::debug!("Altered shipments: {:#?}", pr.altered_shipments);
    tracing::debug!("New Purchase order: {:#?}", pr.purchase_order);

    // 3. Get pending items from database
    let mut tx = pool.begin().await?;
    let mut pending = needs.variant.get_pending_purchase(&mut tx).await?;
    let mut material_shipments = Vec::<MaterialShipment>::new();

    //4. Link pending items to the altered existing shipments
    for ship in pr.altered_shipments {
        let items_to_insert = pending
            .iter()
            .filter(|p| p.due_date == needs.due_date)
            .take(ship.added as usize)
            .map(|p| MaterialShipment::new(p.item_id, ship.id))
            .collect::<Vec<_>>();

        pending.retain(|p| {
            !items_to_insert
                .iter()
                .any(|i| i.raw_material_id() == p.item_id)
        });

        material_shipments.extend(items_to_insert);
    }

    // 5. Insert new shipments into de dabase to get their IDs
    // 6. Link remaining pending items to new purchase orders
    let ship_id = if let Some(shipment) = pr.purchase_order {
        let id = shipment.insert(&mut tx).await?;

        let items_to_insert = pending
            .iter()
            .filter(|p| p.due_date == needs.due_date)
            .take(needs.net_req as usize)
            .map(|p| MaterialShipment::new(p.item_id, id))
            .collect::<Vec<_>>();

        pending.retain(|p| {
            !items_to_insert
                .iter()
                .any(|i| i.raw_material_id() == p.item_id)
        });
        material_shipments.extend(items_to_insert);
        Some(id)
    } else {
        None
    };

    // 7. Insert the new populate the material shipments join table
    //    with the new tuples
    for ms in material_shipments {
        ms.insert(&mut tx).await?;
    }

    // 8. Check if the new shipment has items allocated to it else delete it
    if let Some(id) = ship_id {
        let count = MaterialShipment::count_by_shipment_id(id, &mut tx).await?;
        if count == 0 {
            Shipment::delete(id, &mut tx).await?;
            tracing::warn!("Deleted shipment with id: {}", id);
        }
    }

    tx.commit().await?;

    Ok(())
}

struct QueryResults {
    pub shipments: Vec<UnderAllocatedShipment>,
    pub suppliers: Vec<Supplier>,
}

async fn query_needed_data(
    pool: &PgPool,
    variant: RawMaterial,
    due_date: i32,
) -> anyhow::Result<QueryResults> {
    let mut conn = pool.acquire().await.unwrap();

    let suppliers = Supplier::get_by_item_kind(variant, &mut conn).await?;

    let shipments =
        Shipment::get_under_allocated(due_date, variant, &mut conn).await?;

    let query_results = QueryResults {
        shipments,
        suppliers,
    };

    Ok(query_results)
}

// TODO: Take warehouse capacity into account
// Test if underallocated shipments are being processed correctly
pub async fn resolve_material_needs(
    variant: RawMaterial,
    pool: PgPool,
) -> anyhow::Result<()> {
    tracing::info!("Processing {:?} needs", variant);

    let net_req = {
        let mut conn = pool.acquire().await.unwrap();
        variant.get_net_requirements(&mut conn).await
    }?;

    if net_req.is_empty() {
        tracing::info!("No {:#?} needs at the moment", variant);
        return Ok(());
    }

    tracing::info!(
        "Net {:#?} requirements ({{day: ammount}}): {:?}",
        variant,
        net_req
    );

    for (day, quantity) in net_req.iter() {
        // 1. Get net requirements for the variant by day,
        //    Get under alocated incomming shipments
        //    Get available suppliers
        let qr = query_needed_data(&pool, variant, *day).await?;
        tracing::trace!("{:#?} suppliers: {:?}", variant, qr.suppliers);
        tracing::trace!("Under allocated shipments: {:?}", qr.shipments);

        let needs_data = DayVariantNeedsData {
            due_date: *day,
            net_req: *quantity,
            variant,
            under_allocated: qr.shipments,
            suppliers: qr.suppliers,
        };
        resolve_day_needs(&pool, needs_data).await?;
    }

    tracing::info!("Resolved {:#?} needs", variant);
    Ok(())
}

struct PurchaseProcessingResults {
    pub purchase_order: Option<Shipment>,
    pub altered_shipments: Vec<AlteredShipment>,
}

#[derive(Debug)]
struct AlteredShipment {
    pub id: i64,
    pub added: i64,
}

fn process_purchases(
    needs: &mut DayVariantNeedsData,
    current_date: u32,
) -> PurchaseProcessingResults {
    // 1. Remove from net requirements the stock already ordered in the past
    process_under_alocated_shipments(
        &mut needs.net_req,
        &mut needs.under_allocated,
    );

    // 2. retain only days with net requirements
    //    retain only under allocated shipments to which stock
    //    was allocated and need to be updated on the database
    tracing::trace!(
        "Net requirements after shipment adjusts: {:?}",
        needs.net_req
    );

    let altered_shipments = needs
        .under_allocated
        .iter()
        .map(|s| AlteredShipment {
            id: s.id,
            added: s.added.expect("added is always Some"),
        })
        .collect::<Vec<AlteredShipment>>();

    // 3. Create a purchase order for each supplier for each day
    // Fill low demand days with extra to reach the minimum order quantity.
    let available_time = needs.due_date - current_date as i32;

    if available_time < 0 {
        tracing::warn!("Material for day {} is already due", needs.due_date);
        return PurchaseProcessingResults {
            purchase_order: None,
            altered_shipments,
        };
    }

    let suppliers = needs.suppliers.clone();
    let cheapest_purchase = suppliers
        .into_iter()
        .filter_map(|s| match s.can_deliver_in(available_time) {
            true => Some(s.shipment(needs.net_req, needs.due_date)),
            false => None,
        })
        .min_by_key(|shipment| shipment.cost().0);

    if cheapest_purchase.is_none() {
        tracing::warn!(
            "No supplier can deliver {:#?} in time for day {}",
            needs.variant,
            needs.due_date
        );
    }

    PurchaseProcessingResults {
        purchase_order: cheapest_purchase,
        altered_shipments,
    }
}

fn process_under_alocated_shipments(
    net_req: &mut i32,
    under_allocated: &mut Vec<UnderAllocatedShipment>,
) {
    for s in under_allocated.iter_mut() {
        if *net_req == 0 {
            break;
        }

        let allocated = s.extra_quantity.min(*net_req as i64);
        *net_req -= allocated as i32;
        s.extra_quantity -= allocated;
        s.added = Some(allocated);

        tracing::info!(
            "Allocated {} free slot from shipment id {}",
            allocated,
            s.id,
        );
    }

    // remove shipments to which nothing was allocated
    under_allocated.retain(|s| s.added.is_some());
}
