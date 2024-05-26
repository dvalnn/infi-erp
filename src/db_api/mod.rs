// Modules
mod clients;
mod items;
mod orders;
mod pieces;
mod recipes;
mod shipments;
mod suppliers;
mod transformations;

// Re-exports
pub use clients::*;
pub use items::*;
pub use orders::*;
pub use pieces::*;
pub use recipes::*;
pub use shipments::*;
pub use suppliers::*;
pub use transformations::*;

use sqlx::PgConnection;

pub async fn get_date(con: &mut PgConnection) -> sqlx::Result<u32> {
    Ok(
        sqlx::query_scalar!("SELECT simulation_date FROM epoch_table")
            .fetch_one(con)
            .await? as u32,
    )
}

pub async fn update_date(
    new_date: u32,
    con: &mut PgConnection,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE epoch_table SET simulation_date = $1",
        new_date as i32
    )
    .execute(con)
    .await?;

    Ok(())
}

pub enum NotificationChannel {
    NewOrder,
    MaterialsNeeded,
}

impl NotificationChannel {
    const NEW_ORDER_CHANNEL: &'static str = "new_order";
    const MATERIALS_NEEDED_CHANNEL: &'static str = "materials_needed";

    pub async fn notify(
        channel: NotificationChannel,
        payload: &str,
        con: &mut sqlx::PgConnection,
    ) -> sqlx::Result<()> {
        let query = format!("NOTIFY {}, '{}'", channel, payload);
        sqlx::query(&query).execute(con).await?;
        Ok(())
    }
}

impl std::fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NotificationChannel as Nc;
        match self {
            Nc::NewOrder => write!(f, "new_order"),
            Nc::MaterialsNeeded => write!(f, "materials_needed"),
        }
    }
}

impl TryFrom<&str> for NotificationChannel {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            NotificationChannel::NEW_ORDER_CHANNEL => {
                Ok(NotificationChannel::NewOrder)
            }
            NotificationChannel::MATERIALS_NEEDED_CHANNEL => {
                Ok(NotificationChannel::MaterialsNeeded)
            }
            _ => Err(anyhow::anyhow!("Invalid channel name")),
        }
    }
}
