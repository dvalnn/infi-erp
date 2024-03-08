use sqlx::PgPool;

use crate::db_api::{Item, Order};

pub async fn gen_items(
    order: Order,
    pool: &PgPool,
) -> anyhow::Result<Vec<i64>> {
    let order_id = match order.id() {
        Some(id) => id,
        None => anyhow::bail!("Invalid order id"),
    };

    let mut tx = pool.begin().await?;

    // generate items from order
    // insert items to db
    let mut ids = Vec::new();
    for _ in 0..order.quantity() {
        let item_id = Item::new(order.piece())
            .assign_to_order(order_id)
            .insert(&mut tx)
            .await?;

        ids.push(item_id);
    }

    tracing::info!("Created items: {:?} linked to order {:?}", ids, order_id);

    tx.commit().await?;

    Ok(ids)
}
