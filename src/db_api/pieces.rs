use std::collections::BTreeMap;

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use subenum::subenum;
use uuid::Uuid;

use super::ItemStatus;

#[subenum(FinalPiece, InterPiece, RawMaterial(derive(Sequence)))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "piece_kind")]
pub enum PieceKind {
    #[subenum(RawMaterial)]
    P1,
    #[subenum(RawMaterial)]
    P2,
    #[subenum(InterPiece)]
    P3,
    #[subenum(InterPiece)]
    P4,
    #[subenum(FinalPiece)]
    P5,
    #[subenum(FinalPiece)]
    P6,
    #[subenum(FinalPiece)]
    P7,
    #[subenum(InterPiece)]
    P8,
    #[subenum(FinalPiece)]
    P9,
}

impl std::fmt::Display for FinalPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinalPiece::P5 => write!(f, "P5"),
            FinalPiece::P6 => write!(f, "P6"),
            FinalPiece::P7 => write!(f, "P7"),
            FinalPiece::P9 => write!(f, "P9"),
        }
    }
}

impl TryFrom<&str> for FinalPiece {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "P5" => Ok(FinalPiece::P5),
            "P6" => Ok(FinalPiece::P6),
            "P7" => Ok(FinalPiece::P7),
            "P9" => Ok(FinalPiece::P9),
            _ => Err("Invalid work piece"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawMaterialDetails {
    pub item_id: Uuid,
    pub order_id: Uuid,
    pub due_date: i32,
}

impl RawMaterial {
    pub async fn get_net_requirements(
        &self,
        con: &mut PgConnection,
    ) -> sqlx::Result<BTreeMap<i32, i32>> {
        Ok(sqlx::query!(
            r#"
            SELECT COUNT(items.id) as quantity, transformations.date as date
            FROM items
            JOIN transformations ON items.id = transformations.material_id
            LEFT JOIN raw_material_shipments ON items.id = raw_material_shipments.raw_material_id
            WHERE items.status = $1
                AND items.piece_kind = $2
                AND transformations.date IS NOT NULL
                AND raw_material_shipments.raw_material_id IS NULL
            GROUP BY transformations.date
            "#,
            ItemStatus::Pending as ItemStatus,
            *self as RawMaterial,
        )
        .fetch_all(con)
        .await?
        .into_iter()
        .fold(BTreeMap::new(), |mut map, row| {
            map.insert(
                row.date.expect("selecting only non null"),
                row.quantity.expect("selecting only non null") as i32,
            );
            map
        }))
    }

    pub async fn get_pending_purchase(
        &self,
        con: &mut PgConnection,
    ) -> sqlx::Result<Vec<RawMaterialDetails>> {
        Ok(sqlx::query!(
            r#"
            SELECT
                items.id as item_id,
                items.order_id as order_id,
                transformations.date as due_date
            FROM items
            JOIN transformations ON items.id = transformations.material_id
            LEFT JOIN raw_material_shipments ON items.id = raw_material_shipments.raw_material_id
            WHERE items.status = $1
                AND items.piece_kind = $2
                AND items.order_id IS NOT NULL
                AND transformations.date IS NOT NULL
                AND raw_material_shipments.raw_material_id IS NULL  -- Exclude shiped items
            ORDER BY transformations.date
            "#,
            ItemStatus::Pending as ItemStatus,
            *self as RawMaterial,
        )
        .fetch_all(con)
        .await?
        .iter()
        .map(|row| RawMaterialDetails {
            item_id: row.item_id,
            order_id: row.order_id.expect("selecting only non null"),
            due_date: row.due_date.expect("selecting only non null"),
        })
        .collect())
    }
}
