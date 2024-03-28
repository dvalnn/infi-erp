use sqlx::PgConnection;
use uuid::Uuid;

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
