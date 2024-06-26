use serde::Serialize;
use sqlx::PgConnection;
use sqlx::PgPool;
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
                r#"INSERT INTO transformations
                    (material_id, product_id, recipe_id, date)
                VALUES ($1, $2, $3, $4)
                RETURNING id"#,
                self.material_id,
                self.product_id,
                self.recipe_id,
                self.date,
            )
            .fetch_one(con)
            .await?
            .id,
        );

        Ok(())
    }

    pub async fn complete(
        &self,
        completion_date: u32,
        line: &str,
        machine: &str,
        time_taken: i32,
        con: &mut PgConnection,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"UPDATE transformations
            SET
                status = 'completed',
                date = $1,
                line = $2,
                machine = $3,
                time_taken = $4
            WHERE id = $5"#,
            completion_date as i32,
            line,
            machine,
            time_taken,
            self.id
        )
        .execute(con)
        .await?;
        Ok(())
    }

    pub async fn get_n_next_raw_mat_transf(
        n: i64,
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<Uuid>> {
        Ok(sqlx::query!(
            r#"
            SELECT t.material_id FROM transformations AS t
            JOIN items AS i ON t.material_id = i.id
            WHERE
                (i.piece_kind = 'P1' OR i.piece_kind = 'P2')
                AND i.status = 'in_stock'
                AND t.status = 'pending'
            ORDER BY date
            LIMIT $1;
            "#,
            n
        )
        .fetch_all(con)
        .await?
        .iter()
        .map(|row| row.material_id)
        .collect())
    }

    pub async fn get_by_id(
        id: i64,
        con: &mut PgConnection,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Transformation,
            r#"
            SELECT
                id,
                material_id,
                product_id,
                recipe_id,
                date
            FROM transformations
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(con)
        .await
    }

    async fn query_related_upstream(
        starting_product_id: Uuid,
        pool: &PgPool,
    ) -> sqlx::Result<Vec<Self>> {
        let mut query_id = starting_product_id;
        let mut found = Vec::new();

        while let Some(related) = sqlx::query_as!(
            Transformation,
            "SELECT id, material_id, product_id, recipe_id, date
            FROM transformations WHERE material_id = $1",
            query_id
        )
        .fetch_optional(pool)
        .await?
        {
            query_id = related.product_id;
            found.push(related);
        }

        Ok(found)
    }

    async fn query_related_downstream(
        starting_material_id: Uuid,
        pool: &PgPool,
    ) -> sqlx::Result<Vec<Self>> {
        let mut query_id = starting_material_id;
        let mut found = Vec::new();

        while let Some(related) = sqlx::query_as!(
            Transformation,
            "SELECT id, material_id, product_id, recipe_id, date
            FROM transformations WHERE product_id = $1",
            query_id
        )
        .fetch_optional(pool)
        .await?
        {
            query_id = related.product_id;
            found.push(related);
        }

        Ok(found)
    }

    #[allow(dead_code)]
    pub async fn get_related(
        id: i64,
        pool: &PgPool,
    ) -> sqlx::Result<Vec<Self>> {
        let starting_transf = {
            let mut con = pool.acquire().await?;
            Self::get_by_id(id, &mut con).await?
        };

        let upstream =
            Self::query_related_upstream(starting_transf.product_id, pool)
                .await?;
        let downstream =
            Self::query_related_downstream(starting_transf.material_id, pool)
                .await?;

        let mut related = Vec::new();
        related.extend(upstream);
        related.extend(downstream);

        Ok(related)
    }

    pub fn material_id(&self) -> Uuid {
        self.material_id
    }

    pub fn product_id(&self) -> Uuid {
        self.product_id
    }
}

#[derive(Debug, Serialize)]
pub struct TransformationDetails {
    pub transformation_id: i64,
    pub material_id: Uuid,
    pub product_id: Uuid,
    pub material_kind: PieceKind,
    pub product_kind: PieceKind,
    pub tool: ToolType,
    pub operation_time: i64,
}

impl TransformationDetails {
    pub async fn get_pending_by_day(
        day: i32,
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<TransformationDetails>> {
        sqlx::query_as!(
            TransformationDetails,
            r#"
            SELECT

            transformations.id as transformation_id,
            transformations.material_id,
            transformations.product_id,

            recipes.material_kind as "material_kind: PieceKind",
            recipes.product_kind as "product_kind: PieceKind",
            recipes.tool as "tool: ToolType",
            recipes.operation_time

            FROM transformations

            JOIN recipes ON transformations.recipe_id = recipes.id

            WHERE transformations.date = $1 AND transformations.status = 'pending'
            "#,
            day
        )
        .fetch_all(con)
        .await
    }

    pub async fn get_by_id(
        id: Uuid,
        con: &mut PgConnection,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            TransformationDetails,
            r#"
            SELECT

            t.id as transformation_id,
            t.material_id,
            t.product_id,

            recipes.material_kind as "material_kind: PieceKind",
            recipes.product_kind as "product_kind: PieceKind",
            recipes.tool as "tool: ToolType",
            recipes.operation_time

            FROM transformations AS t

            JOIN recipes ON t.recipe_id = recipes.id

            WHERE t.material_id = $1
            "#,
            id
        )
        .fetch_optional(con)
        .await
    }
}
