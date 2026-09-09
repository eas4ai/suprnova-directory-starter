use crate::notifications::delivery;
use async_trait::async_trait;
use suprnova::{Command, FrameworkError, TypedCommand, serde_json};
#[derive(clap::Parser, Command)]
#[console(
    name = "notifications:deliver",
    description = "Deliver retained owner notifications or inspect pending delivery"
)]
pub struct NotificationsDeliver {
    #[arg(long, default_value = "25", value_parser = clap::value_parser!(u64).range(1..=100))]
    pub limit: u64,
    #[arg(long, conflicts_with = "retry")]
    pub status: bool,
    /// Explicitly retry one failed intent, renewing its bounded attempt budget.
    #[arg(long)]
    pub retry: Option<String>,
}
#[async_trait]
impl TypedCommand for NotificationsDeliver {
    async fn run(self) -> Result<(), FrameworkError> {
        let rows = if self.status {
            delivery::status(self.limit).await?
        } else if let Some(id) = self.retry {
            vec![delivery::retry(&id).await?]
        } else {
            delivery::pending(self.limit).await?
        };
        println!(
            "{}",
            serde_json::to_string(&rows)
                .map_err(|_| FrameworkError::internal("Could not encode delivery status."))?
        );
        if !self.status && rows.iter().any(|row| row.last_error.is_some()) {
            return Err(FrameworkError::bad_request(
                "Some notifications remain undelivered. Inspect their IDs and delivery configuration before retrying.",
            ));
        }
        Ok(())
    }
}
