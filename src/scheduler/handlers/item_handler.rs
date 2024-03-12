use crate::db_api::{Item, RawMaterial, Recipe, Transformation};

pub async fn gen_transformations(
    full_recipe: &[Recipe],
    item: Item,
) -> anyhow::Result<Vec<(Item, Transformation, Recipe)>> {
    let Some(last_step) = full_recipe.last() else {
        anyhow::bail!("No recipes to follow");
    };

    match RawMaterial::try_from(last_step.material_kind) {
        Ok(_) => (),
        Err(_) => tracing::warn!("full recipe is not exaustive!"),
    }

    let mut item_tf_pairs = Vec::new();

    for recipe in full_recipe {
        let material =
            Item::new(recipe.material_kind).set_order(item.order_id());

        // let tf = Transformation::new(product_id, new_id);

        // item_tf_pairs.push((material, tf, *recipe));
    }

    tracing::debug!("new item_tf_pairs: {:#?}", item_tf_pairs);

    Ok(item_tf_pairs)
}
