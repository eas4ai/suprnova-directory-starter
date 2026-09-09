# Starter foundation

Status: Draft
Prefix: FND

These proposed requirements define the first independently verifiable commitment. They do not claim the existing scaffold already meets them. Mechanisms are declared now and implemented during this commitment.

[FND-001] The starter MUST build both Rust binaries against Suprnova v1.3.7 from committed dependency locks.
Falsifier: A clean dependency install changes the lockfile, resolves another Suprnova tag, or fails to build either binary.
Mechanism: foundation-build; locked Cargo build, version assertion and frontend frozen install/build in a disposable checkout.

[FND-002] The starter MUST initialize its account schema on a disposable database using the documented migration command.
Falsifier: The documented command fails on an empty test database, or repeating it fails or changes the completed schema unexpectedly.
Mechanism: foundation-database; isolated migration test with first-run and repeat-run assertions.

[FND-003] The starter MUST enforce account authentication through Suprnova for registration, login, logout, email verification and password reset.
Falsifier: A valid account journey fails, invalid or expired reset/verification tokens succeed, logout leaves the protected session usable, or an unauthenticated request reaches the account dashboard.
Mechanism: foundation-accounts; HTTP account-flow tests against an isolated database, including success, invalid credentials, token expiry and post-logout access.

[FND-004] The starter MUST render public and permission-gated administration shells using Vue, Inertia and Vuetify 0 with shared semantic theme tokens.
Falsifier: Either shell fails to render, an ordinary account reaches the administration shell, keyboard navigation cannot operate its navigation/dialog controls, or changing a shared branding token affects only one shell.
Mechanism: foundation-ui; browser journeys for both shells, role/permission denial checks, keyboard interactions and controlled shared-token change.

[FND-005] The starter MUST document a fresh-checkout setup that reaches the running foundation using declared prerequisites and example configuration.
Falsifier: Following the guide in a disposable checkout requires an undocumented step, a real operator secret, or changes to the original checkout's database.
Mechanism: foundation-setup; execute the README sequence with disposable configuration and record each command and outcome.
