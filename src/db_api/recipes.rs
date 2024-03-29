use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::PieceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[sqlx(type_name = "tool_type")]
pub enum ToolType {
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Recipe {
    pub id: i64,
    pub material_kind: PieceKind,
    pub product_kind: PieceKind,
    pub tool: ToolType,
    pub operation_time: i64,
}

impl Recipe {
    pub async fn get_by_product(
        product: PieceKind,
        pool: &PgPool,
    ) -> sqlx::Result<Vec<Recipe>> {
        sqlx::query_as!(
            Recipe,
            r#"SELECT
                id,
                material_kind as "material_kind: PieceKind",
                product_kind as "product_kind: PieceKind",
                tool as "tool: ToolType",
                operation_time
            FROM recipes WHERE product_kind = $1"#,
            product as PieceKind,
        )
        .fetch_all(pool)
        .await
    }
}
