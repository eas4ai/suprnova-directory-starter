# Suprnova 4.2 compatibility upgrade

Status: Agreed 2026-10-17
Prefix: FWU

Upgrade the existing directory without changing its publication, payment or
permission policies. Local acceptance uses disposable SQLite databases and
provider fixtures. Existing operator data is never a test input.

[FWU-001] The starter MUST build both binaries and client/SSR bundles from committed locks using Suprnova tag v4.2.0 at revision `6f6f83feae1dfde97d614d083625872987407780`, matching payment adapters and a published compatible Markdown plugin with exactly one Suprnova dependency identity.
Falsifier: A locked build changes a lockfile, fails, selects another framework commit, resolves multiple framework identities, or needs an unpublished local dependency override.
Mechanism: framework-upgrade-build
Status: Agreed 2026-10-17

[FWU-002] The starter MUST migrate both an empty database and a populated 3.2.1 database to the supported 4.2 session and RBAC schemas without losing account identifiers, permissions, content or payment history, and repeating migration MUST be harmless.
Falsifier: Either migration fails, preserved fixture records differ, a formerly authorized role loses its intended permissions, an unrelated role gains access, operator grant/revoke fails, or a repeated migration changes completed schema or data.
Mechanism: framework-upgrade-migration
Status: Agreed 2026-10-17

[FWU-003] The starter MUST preserve account suspension and framework authentication protections across the upgrade, including credential rehash forwarding, CSRF rotation and rejection of sessions made stale by a password change.
Falsifier: A suspended account signs in or mutates protected data, the wrapper suppresses a required credential rehash, a pre-rotation CSRF token authorizes a mutation, the current token cannot authorize it, or an old session remains authenticated after password change.
Mechanism: framework-upgrade-auth
Status: Agreed 2026-10-17

[FWU-004] The starter MUST serve its client styles and SSR content through the 4.2 asset configuration in development and production, and MUST prevent shared caching of personalized or token-bearing responses.
Falsifier: A built page lacks required CSS or initial public content/metadata, a production page points to an absent development server, Vite hot-file operation fails, personalized/token-bearing content is publicly cacheable, or an intentionally public cacheable response issues a session/XSRF cookie.
Mechanism: framework-upgrade-http
Status: Agreed 2026-10-17

[FWU-005] The console MUST preserve framework command exit codes and keep help and version usable without application services or built frontend assets.
Falsifier: A usage error returns success or the wrong framework exit code, an unknown command runs bootstrap, or help/version require a database connection or frontend manifest.
Mechanism: framework-upgrade-console
Status: Agreed 2026-10-17

[FWU-006] The upgrade MUST preserve all 55 previously delivered acceptance requirements and provide accurate upgrade, backup, worker and rollback instructions for the final dependency pins.
Falsifier: An inherited delivered requirement fails its mechanism, the guide contradicts the final configuration, omits required durable auth-revocation retry operation, recommends destructive reinitialization, or describes untested external services as verified.
Mechanism: framework-upgrade-regression
Status: Agreed 2026-10-17

## Delivery boundaries

The eight TRF requirements remain planned. Optional framework search,
recommendations, Sentry and a new security-header/CSP policy are excluded.
The developer confirmed this contract and the Markdown prerequisite on
2026-10-17. Markdown v0.4.0 is verified and published at
`6f27ecae53babc0fd637c23b41bc5b244543c78e`; pin that revision here.
It selects the same Suprnova `tag = "v4.2.0"` as the application and adapters.

Retain signed application user identifiers. Verify old RBAC grants and both
direct and role-derived permissions against the migrated layout. Test with a
fixture created by the old schema, not a fresh 4.2 schema labeled as an upgrade.
If sessions must be invalidated by the supported migration, document sign-in
again explicitly; do not promise that old cookies remain usable.

A restore uses the matching pre-upgrade code, database, media and application
key backup. Do not promise old binaries can run against the migrated database.
The final guide must identify the durable queue/worker configuration needed by
framework auth-revocation retries without replacing application notification
policy. Existing acceptance commands remain responsible for their named
requirements; new checks supplement them.
