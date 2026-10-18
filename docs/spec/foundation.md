# Starter foundation

Status: Agreed 2026-09-08
Prefix: FND

These foundation requirements remain acceptance obligations for maintenance upgrades. FND-001 selects the framework revision approved for the current upgrade.

[FND-001] The starter MUST build both Rust binaries against Suprnova tag `v4.2.0` at revision `6f6f83feae1dfde97d614d083625872987407780` from committed dependency locks.
Falsifier: A clean dependency install changes the lockfile, resolves another Suprnova revision, or fails to build either binary.
Mechanism: foundation-build
Status: Agreed 2026-10-17

[FND-002] The starter MUST initialize its account schema on a disposable database using the documented migration command.
Falsifier: The documented command fails on an empty test database, or repeating it fails or changes the completed schema unexpectedly.
Mechanism: foundation-database
Status: Agreed 2026-09-08

[FND-003] The starter MUST enforce account authentication through Suprnova for registration, login, logout, email verification and password reset.
Falsifier: A valid account journey fails, invalid or expired reset/verification tokens succeed, logout leaves the protected session usable, or an unauthenticated request reaches the account dashboard.
Mechanism: foundation-accounts
Status: Agreed 2026-09-08

[FND-004] The starter MUST render public and permission-gated administration shells using Vue, Inertia and Vuetify 0 with shared semantic theme tokens.
Falsifier: Either shell fails to render, an ordinary account reaches the administration shell, keyboard navigation cannot operate its navigation/dialog controls, or changing a shared branding token affects only one shell.
Mechanism: foundation-ui
Status: Agreed 2026-09-08

[FND-005] The starter MUST document a fresh-checkout setup that reaches the running foundation using declared prerequisites and example configuration.
Falsifier: Following the guide in a disposable checkout requires an undocumented step, a real operator secret, or changes to the original checkout's database.
Mechanism: foundation-setup
Status: Agreed 2026-09-08
