# Upgrade Guide

## Directory starter 1.0.0 → 1.0.1

Release `v1.0.1` upgrades Suprnova from 2.0.0 to 3.2.1. Rust and frontend versions are 1.0.1. The framework, Stripe and Paddle adapters all pin `2bd4bd53d04fa4581fdbb152ebd70cfb682b342c`. The verified Markdown pin is `565c2205b1d1d77aa05a29247f3a2416abc60791`; preserve it to reproduce this starter release. The plugin's later `v0.1.1` tag is a separate consumer update.

## Prepare and merge

Preserve `.env`, `APP_KEY`, `DATABASE_URL`, `DIRECTORY_MEDIA_ROOT`, provider credentials and price mappings. Stored billing credentials require the matching encryption key. Do not replace an existing environment with the fresh-install example or regenerate its key.

Commit local changes and retain previous binaries and client/SSR artifacts. Stop web writers and scheduled payment/notification workers. Take a consistent database, media and key backup using the [backup/restore procedure](README.md#production-operation); use database-native tooling for PostgreSQL. Copying a live SQLite file can miss WAL data.

Configure a fork's `upstream` once to `https://github.com/eas4ai/suprnova-directory-starter.git`, then:

```bash
git fetch upstream refs/tags/v1.0.1
git switch -c upgrade/suprnova-3.2.1
git merge --no-ff FETCH_HEAD
```

Retain custom domain code and the compatible dependency pins. Custom RBAC seeds must supply native `chrono::DateTime<Utc>` timestamps with `chrono::Utc::now()`, rather than `.to_rfc3339()`. The release corrects `src/accounts/roles.rs`.

## Build and migrate

Use Rust 1.94+, Bun 1.4.1 and Node 24. From the repository root:

```bash
(cd frontend && bun install --frozen-lockfile && bun run check && bun run build && bun run build:ssr)
cargo build --locked --release --bins
```

Deploy `public/assets/` and `frontend/bootstrap/ssr/` with the binaries. This release adds no application migration file. While writers remain stopped, run pending migrations once using the deployment database and existing key:

```bash
./target/release/directory migrate
```

Start the private SSR worker and web server as separate supervised processes from the application directory:

```bash
node --env-file=.env frontend/bootstrap/ssr/ssr.js
```

```bash
./target/release/directory serve --no-migrate
```

The SSR process runs continuously; keep its port private.

## Production configuration and recovery

Keep `APP_ENV=production`, `APP_DEBUG=false`, the existing `APP_KEY`, HTTPS `APP_URL` and secure-cookie configuration. Persist the database and private media outside replaceable release directories.

Use `RATE_LIMIT_DRIVER=redis` with `RATE_LIMIT_REDIS_URL` for shared quotas. Exactly one server process may explicitly choose memory with `RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION=true`; counters reset on restart and do not protect replicas.

Use a delivering mail driver. Configure SMTP with `MAIL_SMTP_HOST`, `MAIL_SMTP_PORT`, `MAIL_SMTP_USER`, `MAIL_SMTP_PASS`, `MAIL_FROM` and `MAIL_SMTP_ENCRYPTION=starttls` or `tls`, according to the relay. Production rejects non-delivering drivers and plaintext SMTP by default. Local mail-catcher encryption `none` is not a production setting.

Inspect persisted work before resuming schedules:

```bash
./target/release/console billing:reconcile --status --limit 100
./target/release/console notifications:deliver --status --limit 100
```

After deployment verification, resume the existing schedules for `billing:reconcile --limit 25` and `notifications:deliver --limit 25`. Do not seed demo data or recreate provider configuration as an upgrade step.

## Verify and roll back

Check an existing account, administrator permissions, approved listings, articles, Markdown twins, private-media denial, actual JS/CSS loading and no-JavaScript SSR. Verify mail delivery, Redis, provider mode/mappings and authenticated webhooks in your environment. All 54 release requirements passed with isolated SQLite, local browser/SSR requests and synthetic providers; this does not establish real charges or PostgreSQL deployment behavior.

If validation fails, stop all writers and workers. Restore the previous application, matching client/SSR artifacts and configuration. If migrations changed state, restore the matching database, media and key backup. Inspect credentials and pending work before reopening traffic and resuming schedules; do not replay payment events blindly or assume down migrations reverse data changes.
