use crate::db_api::{Item, RawMaterial, Recipe, Transformation};

pub async fn gen_transformations(
    full_recipe: &[Recipe],
    item: Item,
    pool: &sqlx::PgPool,
) -> anyhow::Result<Vec<(Item, Transformation, Recipe)>> {
    let Some(last_step) = full_recipe.last() else {
        anyhow::bail!("No recipes to follow");
    };

    match RawMaterial::try_from(last_step.material_kind) {
        Ok(_) => (),
        Err(_) => tracing::warn!("full recipe is not exaustive!"),
    }

    let mut item_tf_pairs = Vec::new();
    let mut tx = pool.begin().await?;

    let mut product = item;

    for recipe in full_recipe {
        let Some(product_id) = product.id() else {
            anyhow::bail!("Invalid product (Item with None id)");
        };

        let new_item = Item::new(recipe.material_kind).insert(&mut tx).await?;
        let Some(new_id) = new_item.id() else {
            anyhow::bail!("None id returned from DB (new item insertion)");
        };

        let tf = Transformation::new(product_id, new_id)
            .insert(&mut tx)
            .await?;

        item_tf_pairs.push((product, tf, *recipe));
        product = new_item;
    }

    tx.commit().await?;

    tracing::debug!("new item_tf_pairs: {:#?}", item_tf_pairs);

    Ok(item_tf_pairs)
}
