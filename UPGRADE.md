# Upgrade Guide

## Suprnova Directory Starter 1.1.0 → 1.1.1

Suprnova 4.6.2 and suprnova-markdown 0.5.6, which add to the framework's
search and error surfaces and change nothing a deployed copy of this starter
uses. Pin `suprnova` to `tag = "v4.6.2"` and `suprnova-markdown` to
`tag = "v0.5.6"` in `Cargo.toml`, run `cargo update -p suprnova -p suprnova-markdown`,
and redeploy. Nothing else changes.

## Suprnova 4.3.0 → 4.6.1

Release `v1.1.0` moves the starter from Suprnova 4.3.0, released as `v1.0.2`,
to 4.6.1. Rust and frontend package versions are 1.1.0. Merge the `v1.1.0` tag,
not a branch.

The framework, Stripe and Paddle adapters select tag `v4.6.1`, locked to
`5112862367b76a2ea3875d0135ca4ca1ec602c32`. Markdown v0.5.5 selects tag
`v0.5.5`, locked to `1d2650a315a1f424931f2362355fc8ad9dd28b45`, with the
same framework identity. The Inertia Vue, core and Vite packages use the kit's
`^3.8.0` ranges; `frontend/bun.lock` records their exact versions. Keep both
lockfiles. Do not replace these dependencies with local checkouts.

The targeted security refresh clears all 14 vulnerabilities reported by the old
lock. `cargo audit` now exits zero with five maintenance warnings in transitive
dependencies. They have no compatible newer release in the current graph; the
[verification report](docs/upgrade-4.6.1-verification.md) records each warning,
its dependency path and the final audit output. No vulnerability is ignored.

Suprnova 4.6.1 includes the Live action debug-stack fix and the corrected kit
page budget router and gate registration. It needs no additional application
migration beyond the 4.4 through 4.6 changes described below.

### Prepare and merge

Preserve the existing environment, `APP_KEY`, `DATABASE_URL`,
`DIRECTORY_MEDIA_ROOT`, provider credentials and price mappings. Stop web writers,
scheduled payment/notification workers and queue workers. Retain the previous
binaries and matching frontend/SSR artifacts. Take a consistent database, media
and key backup using [the production procedure](README.md#production-operation).
Do not copy a running SQLite database without its WAL, rotate the key, seed demo
data or run fresh-install commands during an upgrade.

```bash
: "${UPGRADE_REF:?Set UPGRADE_REF to the reviewed 1.1.0 commit or published release tag}"
git fetch upstream "$UPGRADE_REF"
git switch -c upgrade/suprnova-4.6.1
git merge --no-ff FETCH_HEAD
```

Keep your application's directory, billing and editorial changes when resolving
conflicts. The Vue kit files now include the Live root document in
`templates/app.html`, the router wrappers in `cmd/main.rs`, the shared frontend
helpers, and named, same-origin login return paths. Include the new template in source deployments and
build images; Askama compiles it into the binaries. The page resolver loads each page on
demand, with shared Vue/Inertia code split from the entry to meet the page budget.

### Build and migrate

Use the repository's Rust toolchain, Node 24, Bun and Suprnova CLI 4.6.1.
Upgrade an older CLI with
`cargo install --force --git https://github.com/eas4ai/suprnova.git --tag v4.6.1 suprnova-cli`. From the repository
root:

```bash
(cd frontend && bun install --frozen-lockfile && bun run check && bun run build && bun run build:ssr)
cargo build --locked --release --bins
./target/release/directory migrate
```

Deploy both binaries, `public/assets/` and `frontend/bootstrap/ssr/` together. The existing migrations still preserve signed application user
IDs and old RBAC grants. Two appended migrations add the case-insensitive email
index and RenderCache's generation ledger. Before migrating, check for addresses
that differ only in case and merge those accounts deliberately; the index refuses
duplicates. Do not discard an account to force a migration through.

Magnetar also creates its eight agent-auth tables and adds
`auth_tokens.address_digest` on first boot. On SQLite and PostgreSQL its widening
change needs no operator step. On MySQL or MariaDB, the first 4.4-or-later boot
widens five authentication tables and can block writes while copying them. Give
one instance enough time to finish before starting the rest. Resolve any warning
about a custom index leaving a column narrow before resuming sign-ins. Email
lookups now require the exact normalized address, including accents, on those
backends. This starter's supported verification backends remain SQLite and
PostgreSQL; a MySQL deployment needs its own driver and migration validation.

The account provider forwards 4.5's atomic `rehash_password_for_sign_in` hook, so
sign-in can upgrade a hash without reviving credentials changed by a concurrent
reset. Login, registration and recovery normalize email before validation, bound
email to 254 characters and registration names to 255. Login return paths accept
only same-origin paths. Test customized account forms against these boundaries.

The router now installs Live and RenderCache after the HTTP stack, as the Vue kit
does. `live::authorize()` owns the reserved notes-topic rule, and both bootstrap
and the page budget test call it before installing Live routes. This directory
starter does not add the kit's sample notes screens or expose directory/billing
actions as tools. `/mcp` has no tools or identity resolvers; OAuth consent and
agent-account registration remain disabled and need no new signing key.

No route currently declares an Inertia cache policy. If your fork has a cacheable
route, replace opaque shared-data providers/hooks with named shares before
registering `Inertia::cache_page`. Classify every shared prop as public or
per-request, including `auth`, `appearance`, `site`, `lang`, `errors` and `flash`.
Boot now refuses a cacheable route whose shares it cannot enumerate. Do not mark
visitor-specific values public to bypass that check.

Vite-fingerprinted files now use
`public, max-age=31536000, immutable`; other static files use
`public, max-age=3600` unless configured otherwise. Deploy new hashed filenames
with each build and retain old assets while old pages can reference them. Static
asset responses no longer issue session/XSRF cookies without a session mutation.
The default JPEG/lossy WebP quality rises from 70 to 85; existing stored media stays
unchanged and new encodes can be larger. The CloudFront feature is enabled by the
framework defaults but serves no publication until configured.

Regenerate the client declarations after merging custom forms:

```bash
suprnova generate-types
```

Commit the generated Inertia props and tool schemas with the application. This
starter does not register searchable framework models or recommendations, so it
needs no search reindex, recommendation generation or ranking-digest migration.
Its SQL directory/article search keeps its existing behavior. Custom search,
recommendation or graph integrations must follow the 4.4 and 4.5 upgrade chapters
before being enabled. Static publication and event sourcing remain opt-in.

### Verify and roll back

Start the SSR worker and web server with the existing production configuration:

```bash
node --env-file=.env frontend/bootstrap/ssr/ssr.js
./target/release/directory serve --no-migrate
```

Keep the SSR port private. Retain the Redis, mail, queue and payment-worker
configuration described below for the 4.3 upgrade. Verify health, existing account
sign-in and recovery, grants and suspension, listings, articles, Markdown,
protected media, payment reconciliation and real asset loading before reopening
traffic. Run `tests/page_budget.rs` with built assets and the SSR worker; it audits
the production router over the public entry pages, including Live assets.
`scripts/verify-framework-upgrade.mjs` covers build, migration, auth, HTTP, console
and regression behavior. Local checks cannot establish live charges, external
SMTP/Redis, cloud storage or your production database and TLS configuration.

If verification fails, stop all writers and workers. Restore the previous
application with its matching frontend/SSR artifacts and configuration, plus the
matching database, media and key backup. Do not pair old binaries with a migrated
database or assume down migrations restore removed data. Preserve Redis work and
logs; do not let old workers consume new framework jobs or replay payment events
blindly.

## Suprnova 3.2.1 → 4.3.0 (also from 4.2.0)

This upgrade was released as `v1.0.2`; its Rust and frontend package versions
were 1.0.1. The `v1.0.1` tag holds the earlier Suprnova 3.2.1 release, so fetch
`v1.0.2` for the 4.3 code.

The framework, Stripe and Paddle adapters select tag `v4.3.0`, locked to
`2e0c5227b9090c47894169c0d545b40eb27c3641`. Markdown v0.5.0 is pinned to
`c80dafb5f05e6de633472f73beec4d5101883389` and selects the same framework identity.
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
: "${UPGRADE_REF:?Set UPGRADE_REF to the reviewed Suprnova 4.3 starter commit or release tag}"
git fetch upstream "$UPGRADE_REF"
git switch -c upgrade/suprnova-4.3.0
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

The migration chain retains `RbacToSpatieLayout` after the framework RBAC creation
migration, and `CreateSessionsTable::new(SessionUserKey::Integer)` after the
application migrations. When upgrading from 3.2.1, these reshape existing grants
and database sessions. They are already applied on 4.2.0. Do not rename tables
manually or recreate the database. Disposable checks cover empty databases and
populated 3.2.1 and 4.2.0 data, signed account IDs, role-derived and direct
permissions, content/payment records, session payloads and a repeat run. The
4.2.0 baseline was captured with the previous application and framework; this
upgrade preserves its schema and records.

Suprnova 4.3 adds `search_source_versions` for registered searchable models and
new recommendation graph tables. This starter uses its own SQL discovery and
does not register framework searchable models or enable recommendations, so
those migrations and services are not added. A customized installation using
those features must follow the framework 4.3 upgrade guide and register their
migrations before accepting writes. New media embeddings and graph
recommendations remain opt-in.

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
