use sqlx::PgConnection;

#[derive(Debug, Clone, Copy)]
pub struct Transformation {
    id: Option<i64>,
    material_id: i64,
    product_id: i64,
    date: Option<i32>,
}

impl Transformation {
    pub fn new(product_id: i64, material_id: i64) -> Self {
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
