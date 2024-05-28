use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

use super::FinalPiece;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "delivery_line")]
pub enum DeliveryLines {
    DL1,
    DL2,
    DL3,
    DL4,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryStatistics {
    line: DeliveryLines,
    piece: FinalPiece,
    quantity: i32,
    associated_order_id: Uuid,
}

impl DeliveryStatistics {
    pub async fn insert(&self, con: &mut PgConnection) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO delivery_statistics
                (line, piece, quantity, associated_order_id)
            VALUES ($1, $2, $3, $4)
            "#,
            self.line as DeliveryLines,
            self.piece as FinalPiece,
            self.quantity,
            self.associated_order_id
        )
        .execute(con)
        .await?;

        Ok(())
    }
}
