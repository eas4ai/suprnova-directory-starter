//! Shared kit router wiring. Directory routes expose no agent tools or accounts.

use suprnova::agents::AgentsConfig;
use suprnova::{FrameworkError, Router};

/// Mount the MCP protocol without enabling OAuth or agent account registration.
/// Directory and billing actions retain their existing session and permission guards.
pub fn serve(router: Router) -> Result<Router, FrameworkError> {
    router.try_agents(AgentsConfig::new())
}
