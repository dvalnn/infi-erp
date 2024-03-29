use serde::Serialize;
use sqlx::PgConnection;
use uuid::Uuid;

use super::PieceKind;
use super::ToolType;

#[derive(Debug, Clone)]
pub struct Transformation {
    id: Option<i64>,
    material_id: Uuid,
    product_id: Uuid,
    recipe_id: i64,
    date: Option<i32>,
}

impl Transformation {
    pub fn new(product_id: Uuid, material_id: Uuid, recipe_id: i64) -> Self {
        Self {
            id: None,
            material_id,
            product_id,
            recipe_id,
            date: None,
        }
    }

    pub fn set_date(&mut self, date: i32) {
        self.date = Some(date);
    }

    pub async fn insert(&mut self, con: &mut PgConnection) -> sqlx::Result<()> {
        self.id = Some(
            sqlx::query!(
                "INSERT INTO transformations (material_id, product_id, recipe_id, date)
                VALUES ($1, $2, $3, $4)
                RETURNING id",
                self.material_id,
                self.product_id,
                self.recipe_id,
                self.date
            )
            .fetch_one(con)
            .await?
            .id,
        );

        Ok(())
    }

    pub fn material_id(&self) -> Uuid {
        self.material_id
    }

    pub fn product_id(&self) -> Uuid {
        self.product_id
    }

    pub fn date(&self) -> Option<i32> {
        self.date
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }
}

#[derive(Debug, Serialize)]
pub struct TransformationDetails {
    pub transformation_id: i64,
    pub material_kind: PieceKind,
    pub product_kind: PieceKind,
    pub tool: ToolType,
    pub operation_time: i64,
}

impl TransformationDetails {
    pub async fn get_by_date(
        day: i32,
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<TransformationDetails>> {
        sqlx::query_as!(
            TransformationDetails,
            r#"
            SELECT
            transformations.id as "transformation_id",
            recipes.material_kind as "material_kind: PieceKind",
            recipes.product_kind as "product_kind: PieceKind",
            recipes.tool as "tool: ToolType",
            recipes.operation_time
            FROM transformations
            JOIN recipes ON transformations.recipe_id = recipes.id
            WHERE transformations.date = $1
            "#,
            day
        )
        .fetch_all(con)
        .await
    }
}
