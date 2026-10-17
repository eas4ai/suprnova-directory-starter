use async_trait::async_trait;
use suprnova::{Command, FrameworkError, TypedCommand};

#[derive(clap::Parser, Command)]
#[console(
    name = "directory:demo",
    description = "Seed isolated demonstration content without changing existing records"
)]
pub struct DemoSeed {
    /// Explicitly permit demo data outside local, development and test environments.
    #[arg(long)]
    pub allow_production: bool,
}

#[async_trait]
impl TypedCommand for DemoSeed {
    async fn run(self) -> Result<(), FrameworkError> {
        let created = crate::demo::seed(self.allow_production).await?;
        if created {
            println!(
                "Demo content created. Synthetic entitlements are demonstration data; no payment was made. The demo owner has no usable supplied password or administrative permissions."
            );
        } else {
            println!(
                "Demo seed already completed; existing content and credentials were preserved."
            );
        }
        Ok(())
    }
}
