// Included only in the verifier's disposable console source.
#[derive(clap::Parser, suprnova::Command)]
#[console(
    name = "compat:exit",
    description = "Verify framework exit-code forwarding"
)]
pub struct UpgradeExitFixture {}

#[async_trait::async_trait]
impl suprnova::TypedCommand for UpgradeExitFixture {
    async fn run(self) -> Result<(), suprnova::FrameworkError> {
        Err(suprnova::FrameworkError::exit(23))
    }
}
