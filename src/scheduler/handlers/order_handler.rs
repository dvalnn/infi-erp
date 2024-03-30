use anyhow::Result;
use uuid::Uuid;

use crate::db_api::{Item, PieceKind, Recipe};

pub fn gen_items(
    piece: PieceKind,
    quantity: i32,
    id: Option<Uuid>,
) -> Result<Vec<Item>> {
    // generate items from order
    let items = (0..quantity).fold(Vec::new(), |mut acc, _| {
        let item = Item::new(piece).set_order(id);
        acc.push(item);
        acc
    });

    tracing::info!("Created {} items linked to order id {:?}", items.len(), id);
    tracing::trace!("Items: {:?}", items);

    Ok(items)
}

pub(crate) async fn get_full_recipe(
    piece: PieceKind,
    pool: &sqlx::PgPool,
) -> Result<Vec<Recipe>> {
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
