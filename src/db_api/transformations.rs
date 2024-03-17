use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
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

    pub fn set_date(&mut self, date: i32) {
        self.date = Some(date);
    }

    pub async fn insert(&mut self, con: &mut PgConnection) -> sqlx::Result<()> {
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
