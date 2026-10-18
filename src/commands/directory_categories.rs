use async_trait::async_trait;
use suprnova::{Command, FrameworkError, TypedCommand};

#[derive(clap::Parser, Command)]
#[console(
    name = "directory:categories",
    description = "Create starter listing categories without replacing existing terms"
)]
pub struct DirectoryCategories;

#[async_trait]
impl TypedCommand for DirectoryCategories {
    async fn run(self) -> Result<(), FrameworkError> {
        crate::listings::workflow::seed_categories().await?;
        println!("Starter listing categories are available; existing terms were preserved.");
        Ok(())
    }
}
