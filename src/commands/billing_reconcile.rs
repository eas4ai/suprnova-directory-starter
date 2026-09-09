use async_trait::async_trait;
use suprnova::{Command, FrameworkError, TypedCommand, serde_json};

use crate::billing::{gateway, reconcile};

#[derive(clap::Parser, Command)]
#[console(
    name = "billing:reconcile",
    description = "Replay payment evidence or inspect existing provider resources; never creates charges"
)]
pub struct BillingReconcile {
    /// Maximum retained events to process or display (1–100).
    #[arg(long, default_value = "25", value_parser = clap::value_parser!(u64).range(1..=100))]
    pub limit: u64,
    /// Display pending and failed evidence without contacting providers.
    #[arg(long, conflicts_with_all = ["purchase", "event", "use_current_credentials"])]
    pub status: bool,
    /// Recover this exact purchase using authoritative existing provider state.
    #[arg(long, conflicts_with = "event")]
    pub purchase: Option<String>,
    /// Existing session, transaction, invoice or subscription ID; never a URL.
    #[arg(long, requires = "purchase")]
    pub resource: Option<String>,
    /// Retry one retained event, including an exhausted attempt budget.
    #[arg(long)]
    pub event: Option<String>,
    /// Explicit recovery after API-key replacement. Uses only read operations.
    #[arg(long)]
    pub use_current_credentials: bool,
}

#[async_trait]
impl TypedCommand for BillingReconcile {
    async fn run(self) -> Result<(), FrameworkError> {
        if self.use_current_credentials && self.purchase.is_none() && self.event.is_none() {
            return Err(FrameworkError::bad_request(
                "Scope --use-current-credentials to --purchase or --event.",
            ));
        }
        if self.status {
            return output(&reconcile::status(self.limit).await?);
        }
        let gateway = gateway::service()?;
        let rows = if let Some(id) = self.purchase {
            vec![
                reconcile::recover(
                    &id,
                    self.resource.as_deref(),
                    gateway.as_ref(),
                    self.use_current_credentials,
                )
                .await?,
            ]
        } else if let Some(id) = self.event {
            vec![
                reconcile::process_event(&id, gateway.as_ref(), true, self.use_current_credentials)
                    .await?,
            ]
        } else {
            reconcile::pending(self.limit, gateway.as_ref()).await?
        };
        output(&rows)?;
        if rows.iter().any(|row| row.error_code.is_some()) {
            return Err(FrameworkError::bad_request(
                "Some payment evidence remains unresolved. Inspect the reported event IDs and error codes; retry after correcting the cause.",
            ));
        }
        Ok(())
    }
}

fn output(value: &impl serde::Serialize) -> Result<(), FrameworkError> {
    println!(
        "{}",
        serde_json::to_string(value)
            .map_err(|_| FrameworkError::internal("Could not encode reconciliation status."))?
    );
    Ok(())
}
