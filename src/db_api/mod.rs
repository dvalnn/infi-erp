#![allow(dead_code, unused_imports)]

// Modules
mod clients;
mod items;
mod orders;
mod pieces;

// Re-exports
pub use clients::*;
pub use items::*;
pub use orders::*;
pub use pieces::*;

pub enum NotificationChannel {
    NewOrder,
    // NewBomEntry,
}

impl NotificationChannel {
    const NEW_ORDER_CHANNEL: &'static str = "new_order";
    const NEW_BOM_ENTRY_CHANNEL: &'static str = "new_bom_entry";
    const ALL_STR: [&'static str; 2] =
        [Self::NEW_ORDER_CHANNEL, Self::NEW_BOM_ENTRY_CHANNEL];

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
            // Nc::NewBomEntry => write!(f, "new_bom_entry"),
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
            // NotificationChannel::NEW_BOM_ENTRY_CHANNEL => {
            //     Ok(NotificationChannel::NewBomEntry)
            // }
            _ => Err(anyhow::anyhow!("Invalid channel name")),
        }
    }
}
