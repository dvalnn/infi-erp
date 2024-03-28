use crate::db_api::{Item, RawMaterial, Recipe, Transformation};

#[derive(Debug)]
pub struct Step {
    pub material: Item,
    pub transf: Transformation,
    pub recipe: Recipe,
}

impl Step {
    pub fn new(material: Item, transf: Transformation, recipe: Recipe) -> Self {
        Self {
            material,
            transf,
            recipe,
        }
    }
}

pub fn describe_process(
    full_recipe: &[Recipe],
    item: Item,
) -> anyhow::Result<Vec<Step>> {
    let Some(last_step) = full_recipe.last() else {
        anyhow::bail!("No recipes to follow");
    };

    match RawMaterial::try_from(last_step.material_kind) {
        Ok(_) => (),
        Err(_) => tracing::warn!("full recipe is not exaustive!"),
    }

    let mut item_tf_pairs = Vec::new();

    let mut product_id = item.id();
    for recipe in full_recipe {
        let mat = Item::new(recipe.material_kind).set_order(item.order_id());
        let transf = Transformation::new(product_id, mat.id(), recipe.id);

        product_id = mat.id();
        item_tf_pairs.push(Step::new(mat, transf, (*recipe).clone()));
    }

    tracing::trace!("new process: {:?}", item_tf_pairs);

    Ok(item_tf_pairs)
}
