use sqlx::PgPool;

use crate::db_api::{Item, Order, PieceKind, Recipe};

pub async fn gen_items(
    order: Order,
    pool: &PgPool,
) -> anyhow::Result<Vec<Item>> {
    let order_id = match order.id() {
        Some(id) => id,
        None => anyhow::bail!("Invalid order id"),
    };

    let mut tx = pool.begin().await?;

    // generate items from order
    // insert items to db
    let mut items = Vec::new();
    for _ in 0..order.quantity() {
        let item = Item::new(order.piece())
            .assign_to_order(order_id)
            .insert(&mut tx)
            .await?;

        items.push(item);
    }

    tracing::info!("Created items: {:?} linked to order {:?}", items, order_id);

    tx.commit().await?;

    Ok(items)
}

pub(crate) async fn gen_full_recipe(
    piece: PieceKind,
    pool: &sqlx::PgPool,
) -> anyhow::Result<Vec<Recipe>> {
    let mut product = piece;
    let mut full_recipe = Vec::new();

    loop {
        let recipes = Recipe::get_by_product(product, pool).await?;
        if recipes.is_empty() {
            return Ok(full_recipe);
        };

        //TODO: implement a better heuristic
        let fastest = match recipes.into_iter().min_by_key(|r| r.operation_time)
        {
            Some(f) => f,
            None => anyhow::bail!("No recipe found for product {:?}", product),
        };

        product = fastest.material_kind;
        full_recipe.push(fastest);
    }
}
