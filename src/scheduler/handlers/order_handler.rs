use anyhow::Result;
use async_recursion::async_recursion;
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

#[async_recursion]
pub(crate) async fn get_full_recipe(
    piece: PieceKind,
    pool: &sqlx::PgPool,
) -> Result<Vec<Recipe>> {
    let recipes = Recipe::get_by_product(piece, pool).await?;
    if recipes.is_empty() {
        return Ok(vec![]);
    }

    let mut possible_paths = Vec::new();
    for recipe in recipes {
        let subrecipe = get_full_recipe(recipe.material_kind, pool).await?;
        let mut recipe_path = vec![recipe];
        recipe_path.extend(subrecipe);
        possible_paths.push(recipe_path);
    }

    let best = possible_paths
        .into_iter()
        .min_by_key(|r| r.iter().map(|r| r.operation_time).sum::<i64>());

    tracing::debug!("Best full recipe for piece {:?}: {:?}", piece, best);

    match best {
        Some(b) => Ok(b),
        None => unreachable!(),
    }
}
