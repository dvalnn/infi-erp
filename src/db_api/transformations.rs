use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct Transformation {
    id: Option<i64>,
    material_id: Uuid,
    product_id: Uuid,
    date: Option<i32>,
}

impl Transformation {
    pub fn new(product_id: Uuid, material_id: Uuid) -> Self {
        Self {
            id: None,
            material_id,
            product_id,
            date: None,
        }
    }

    pub async fn insert(
        mut self,
        con: &mut PgConnection,
    ) -> sqlx::Result<Self> {
        self.id = Some(
            sqlx::query!(
                "INSERT INTO transformations (material_id, product_id)
                VALUES ($1, $2)
                RETURNING id",
                self.material_id,
                self.product_id
            )
            .fetch_one(con)
            .await?
            .id,
        );

        Ok(self)
    }
}
