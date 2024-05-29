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

// #[async_recursion]
// pub(crate) async fn get_full_recipe(
//     piece: PieceKind,
//     pool: &sqlx::PgPool,
// ) -> Result<Vec<Recipe>> {
//     let mut product = piece;
//     let mut full_recipe = Vec::new();
//
//     loop {
//         let recipes = Recipe::get_by_product(product, pool).await?;
//         if recipes.is_empty() {
//             tracing::warn!("Full recipe: {:?}", full_recipe);
//             return Ok(full_recipe);
//         };
//
//         tracing::warn!("Recipes for product{:?}: {:?}", product, recipes);
//
//         if recipes.len() == 1 {
//             product = recipes[0].material_kind;
//             tracing::warn!("Adding recipe to full recipe: {:?}", recipes[0]);
//             full_recipe.push(recipes[0].clone());
//             continue;
//         }
//
//         let mut subrecipes = Vec::new();
//         let n_recursions = recipes.len();
//         for (idx, recipe) in recipes.clone().into_iter().enumerate() {
//             tracing::warn!("RECURSING {}/{}", idx, n_recursions);
//             let subrecipe = get_full_recipe(recipe.material_kind, pool).await?;
//             subrecipes.push(subrecipe);
//         }
//
//         let Some(mut best) = subrecipes
//             .into_iter()
//             .enumerate()
//             .min_by_key(|r| r.1.iter().map(|r| r.operation_time).sum::<i64>())
//         else {
//             tracing::warn!(
//                 "Full recipe returning from place 2: {:?}",
//                 full_recipe
//             );
//             return Ok(full_recipe);
//         };
//
//         best.1.extend(vec![recipes[best.0].clone()]);
//         product = best.1[0].material_kind;
//         full_recipe.extend(best.1);
//     }
// }

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
