# Upgrade Guide

## Suprnova 3.2.1 → 4.2.0

This is the current, unreleased framework upgrade. Rust and frontend package
versions are 1.0.1; no new starter release tag is implied. The existing `v1.0.1`
tag still identifies the earlier Suprnova 3.2.1 release. Select a reviewed commit
containing this upgrade, or a later explicitly published starter release, rather
than fetching that old tag expecting the 4.2 code.

The framework, Stripe and Paddle adapters select tag `v4.2.0`, locked to
`6f6f83feae1dfde97d614d083625872987407780`. Markdown v0.4.0 is pinned to
`6f27ecae53babc0fd637c23b41bc5b244543c78e` and selects the same framework identity.
The Inertia Vue, core and Vite packages are pinned to 3.8.0. Keep both dependency
locks to reproduce the build; do not substitute a local framework checkout.

## Prepare and merge

Preserve `.env`, `APP_KEY`, `DATABASE_URL`, `DIRECTORY_MEDIA_ROOT`, provider
credentials and price mappings. Stored billing credentials require the matching
encryption key. Do not replace an existing environment with the fresh-install
example or regenerate its key.

Commit local changes and retain previous binaries and client/SSR artifacts.
Stop web writers, scheduled payment/notification workers and any queue workers.
Take a consistent database, media and key backup using the
[backup/restore procedure](README.md#production-operation); use database-native
tooling for PostgreSQL. Copying a live SQLite file can miss WAL data. Preserve
Redis queue state and recovery logs separately if already using a durable queue.

Configure a fork's `upstream` once to
`https://github.com/eas4ai/suprnova-directory-starter.git`. Set `UPGRADE_REF` to the
reviewed starter commit or release tag, then:

```bash
: "${UPGRADE_REF:?Set UPGRADE_REF to the reviewed Suprnova 4.2 starter commit or release tag}"
git fetch upstream "$UPGRADE_REF"
git switch -c upgrade/suprnova-4.2.0
git merge --no-ff FETCH_HEAD
```

Retain custom domain code and the pins above. Review customized RBAC queries and
seeds against the new role/permission layout: roles and permissions are guard
scoped, grant pivots use the framework's Spatie-compatible layout, and framework
identifiers are unsigned. This starter retains signed application user IDs.
Custom timestamps remain native `chrono::DateTime<Utc>` values from
`chrono::Utc::now()`, not `.to_rfc3339()` strings.

## Build and migrate

Use the verified Rust 1.95 toolchain, Bun 1.4.3 and Node 24. The package declares
a Rust 1.94 minimum, but this upgrade was checked with 1.95. From the repository
root:

```bash
(cd frontend && bun install --frozen-lockfile && bun run check && bun run build && bun run build:ssr)
cargo build --locked --release --bins
```

Deploy `public/assets/` and `frontend/bootstrap/ssr/` with both binaries. Keep
writers stopped and run the registered migrations once against the deployment
database with its existing key:

```bash
./target/release/directory migrate
```

The upgrade registers `RbacToSpatieLayout` after the framework RBAC creation
migration, and `CreateSessionsTable::new(SessionUserKey::Integer)` after the
application migrations. They reshape existing grants and database sessions; do
not rename tables manually or recreate the database. Disposable checks cover both
empty databases and populated 3.2.1 data, signed account IDs, role-derived and
direct permissions, content/payment records, session payloads and a repeat run.

The session migration preserves stored IDs, payloads and CSRF tokens. Old browser
cookie continuity is not guaranteed by the starter's acceptance checks: plan for
users to sign in again and verify an existing account after deployment. Sessions
made stale by a password change must be rejected. Do not run the old application
against the migrated database.

## Assets and server rendering

The client build and both Inertia configurations explicitly include `src/app.css`.
Production requires the built Vite manifest and ignores `public/hot`. HTML,
Inertia and cookie-bearing responses are sent `private, no-store`; do not override
this at a proxy or CDN. Public Markdown remains subject to current eligibility.

Start the private SSR worker and web server as separate supervised processes from
the application directory:

```bash
node --env-file=.env frontend/bootstrap/ssr/ssr.js
```

```bash
./target/release/directory serve --no-migrate
```

Keep matching `SSR_PORT` and `SSR_URL` settings and keep the renderer port private.
In development, follow the README's Vite subshell: it creates and removes the hot
file, and the Inertia Vite plugin serves SSR from source. Wait for module warm-up.
For a built local preview, stop Vite and start the standalone worker. A bare Vite
process without a hot file does not select hot assets when a manifest exists.

## Production configuration and workers

Keep `APP_ENV=production`, `APP_DEBUG=false`, the existing `APP_KEY`, HTTPS
`APP_URL` and secure-cookie configuration. Persist database and private media
outside replaceable release directories. Leave the application's publication,
moderation, payment and notification policies intact.

Use `RATE_LIMIT_DRIVER=redis` with `RATE_LIMIT_REDIS_URL` for shared quotas.
Exactly one server process may explicitly choose memory with
`RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION=true`; counters reset on restart and do not
protect replicas.

Use a delivering mail driver. Configure SMTP with `MAIL_SMTP_HOST`,
`MAIL_SMTP_PORT`, `MAIL_SMTP_USER`, `MAIL_SMTP_PASS`, `MAIL_FROM` and
`MAIL_SMTP_ENCRYPTION=starttls` or `tls`, according to the relay. Production rejects
non-delivering drivers and plaintext SMTP by default. Local mail-catcher
encryption `none` is not a production setting.

Credential-revocation retries now need durable queue operation. A password change
can commit before session/remember-token cleanup fails. Suprnova queues
`Suprnova::RevokeUserCredentials` to finish cleanup. Configure these settings on
both web and worker processes, using the deployment's private Redis address:

```dotenv
QUEUE_DRIVER=redis
QUEUE_REDIS_URL=redis://your-private-redis:6379/1
QUEUE_REDIS_STREAM=directory-queue
QUEUE_REDIS_GROUP=default
QUEUE_VISIBILITY_TIMEOUT_SECS=120
```

Enable Redis persistence and backups; use a distinct stream per installation.
Leave `QUEUE_REDIS_CONSUMER` unset so processes receive distinct identities.
Memory, sync and null queues do not supply durable retry storage. The queue URL
is separate from the rate-limit URL. This uses the existing Redis driver; this
starter does not register database queue migrations.

Supervise an additional process and restart it with each deployment:

```bash
./target/release/directory queue:work --queue default --visibility-timeout 120
```

Leave `--tries` unset so the revocation job retains unlimited retries, a 60-second
attempt timeout and backoff from about 10 seconds to at most one hour. Monitor
worker errors and the Redis pending/delayed backlog. The starter does not install
a Redis failed-job store; do not rely on `queue:failed` to prove there is no
pending work. If enqueueing fails too, the password may already have changed:
repair the services and finish revocation before closing the incident. Replaying
the spent password-reset link is not recovery.

Inspect the application's separate persisted work before resuming schedules:

```bash
./target/release/console billing:reconcile --status --limit 100
./target/release/console notifications:deliver --status --limit 100
```

After deployment verification, resume the existing schedules for
`billing:reconcile --limit 25` and `notifications:deliver --limit 25`. They do not
replace the framework queue worker. Do not seed demo data or recreate provider
configuration as an upgrade step. The console preserves framework exit codes:
help/version return zero, parse errors return one in this pinned framework, and
commands may return their own nonzero codes.

## Verify and roll back

Check an existing account, administrator grant/revoke, approved listings,
articles, Markdown twins, private-media denial, actual JS/CSS loading and
no-JavaScript SSR. Verify mail delivery, Redis persistence/worker restart,
provider mode/mappings and authenticated webhooks in the deployment environment.

Run the six tasks in `scripts/verify-framework-upgrade.mjs` for local upgrade
acceptance: `build`, `migration`, `auth`, `http`, `console` and `regression`.
The regression task runs all 55 inherited requirements. Local checks use isolated
SQLite, browser/SSR requests and synthetic provider fixtures. They do not establish
PostgreSQL behavior, external SMTP, Redis persistence/outage recovery, cloud
storage, live charges or a production TLS deployment.

If validation fails, stop all writers and workers. Restore the previous
application with its matching client/SSR artifacts and configuration, plus the
matching database, media and key backup from before migration. Never pair old
binaries with the migrated database or assume down migrations reverse data
changes. Preserve queue state and logs for investigation; do not let old workers
consume new framework jobs. Review credential changes and pending work before
reopening traffic. Inspect provider credentials and unresolved payments; do not
replay payment events blindly.
