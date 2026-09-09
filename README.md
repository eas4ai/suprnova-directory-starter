# Suprnova directory starter

A free directory starter built on Suprnova, with account flows and Stripe/Paddle
administration. Listing publication follows in
[the roadmap](docs/spec/roadmap.md).

The framework and both payment adapters are pinned to Suprnova commit
`107e6e7a122d5145160ea1547ca90ddc37459c27` for the reviewed checkout repairs.
Cargo.lock records that exact revision; no local framework checkout is required.

## Prerequisites

- Rust 1.94 or newer and Cargo; this checkout is verified with Rust 1.95.
- Bun 1.4.1 for frontend packages and database inspection.
- Node 24 for local key generation and verification. Cairn is needed only for
  specification lint and evidence recording, not to run the application.
- A C/C++ toolchain, pkg-config and OpenSSL development headers for native dependencies.
- Git and network access for the first dependency installation.

## Install and migrate

Run from a fresh checkout. Keep an existing operator configuration when updating
an installation; the copy command below is only for a new checkout.

<!-- foundation:install -->
```sh
cp .env.example .env
node -e 'const fs=require("node:fs"); const crypto=require("node:crypto"); const p=".env"; fs.writeFileSync(p,fs.readFileSync(p,"utf8").replace(/^APP_KEY=.*$/m,"APP_KEY="+crypto.randomBytes(32).toString("base64url")))'
cargo build --locked --bins
cd frontend
bun install --frozen-lockfile
bun run build
cd ..
cargo run --locked --bin directory -- migrate
```

The default database is `database.db` in this checkout. SQLite requires no database
service. The migration command creates account and billing settings tables and records completed
migrations. Running the same command again applies only pending migrations.
It is Suprnova's application migration entry point; the developer CLI's
`suprnova migrate` delegates to it.

## Run locally

Start Vite in one terminal, from the repository root:

<!-- foundation:vite -->
```sh
cd frontend
bun run dev
```

In a second terminal, from the repository root:

<!-- foundation:serve -->
```sh
cargo run --locked --bin directory -- serve --no-migrate
```

Open `http://localhost:8765`. The public directory and account forms should render.
Keep both terminals running; stop each with Ctrl+C. Local mode loads modules from
Vite on port 5765. `bun run build` checks production assets but does not replace
Vite in local mode. The guide uses the application binaries directly, so the
separately installed Suprnova developer CLI is not required.

If either port is occupied, export `SERVER_PORT`, `VITE_PORT`, and `APP_URL` with
matching free ports in both terminals before running the commands. The backend
reads these before `.env`, and Vite reads `VITE_PORT` from the terminal environment.
The setup verifier uses this documented override with ephemeral local ports.

## Account mail

To register or recover accounts, run a local SMTP capture server on
`localhost:1025`, without authentication or encryption, as configured in
`.env.example`. The server and forms can start without it, but sending account
mail requires it. Use the capture server's inbox to open verification/reset links.
An external SMTP relay must be configured separately before deployment.

Account email uses the SMTP service on port 1025 from the example configuration.
Registration sends a verification link; sign in and open `/verify-email` to resend
it. Open the link while signed in to the account it belongs to. Password reset
at `/forgot-password` sends links only for verified accounts. Unknown and unverified
addresses receive the same public response. Resetting a password revokes existing
sessions and remember tokens. Mail links use `APP_URL`, so set it to the address
where the application is reachable.

## Administrator access

Register an account, verify its email, then read its account ID on **Your account**.
From the application host, replace `42` below with that exact ID:

```sh
cargo run --locked --bin console -- admin:access grant --user-id 42
```

The command grants `admin.access` and `billing.configure` using Suprnova RBAC.
It refuses unknown or unverified accounts. Repeating a grant is safe. Sign in and
open **Administration → Payment providers**. An existing session sees the grant
on its next request; registration itself never grants administrative access.

To revoke access, including access in an existing session:

```sh
cargo run --locked --bin console -- admin:access revoke --user-id 42
```

Revocation removes the two direct permissions and any role memberships granting
either permission. Other role memberships remain. Roles and their permissions
are not deleted. The same host command can recover access if no administrators
remain; no public bootstrap endpoint is exposed.

## Payment provider configuration

Select **Test mode** or **Live mode**, enter a provider's three credentials, enable
it if desired, then save. Both modes start disabled. Stripe needs its secret API
key, publishable key and webhook signing secret. Paddle needs its API key, client
token and webhook key; test mode uses Paddle Sandbox. Use current mode-prefixed
credentials. Legacy Paddle API keys without a mode prefix are not accepted.

API and webhook secrets are encrypted with Suprnova's context-bound AES-GCM under
`APP_KEY`. They are never returned to the browser. Leave replacement fields blank
to preserve saved secrets. To remove credentials, disable the provider and select
**Clear credentials when saved**. Clearing also removes the public client key.
A disabled provider cannot remain the default. Disabling alone preserves its
credentials and mappings.

Database settings are authoritative: `STRIPE_*` and `PADDLE_*` environment variables
do not override them. `APP_KEY` must be generated and explicitly configured even
for local billing administration. The application's temporary local development
key is insufficient. Keep the key in deployment secret storage and back it up
separately from the database. If the key is lost or the stored ciphertext is
damaged, the UI refuses edits; restore the matching key/database and restart.
Changing the key does not automatically migrate existing billing credentials.
This starter does not provide an automatic key-rotation command.

Map each local plan identifier to a Stripe `price_…` and/or Paddle `pri_…` identifier.
Mappings are separate by mode; a blank price has no fallback. The limit is 100
plans per mode. Plan identifiers use 1–64 lowercase letters, digits, hyphens or
underscores; credential and price fields are bounded to 512 characters.
Remove a mapping row and save to delete it. Saving a stale form reports a conflict;
reload the saved settings before applying your changes again.

Saving validates local format and adapter construction. It does **not** contact
Stripe or Paddle, verify authentication or price availability, or create a checkout.
Webhook endpoints and payment processing arrive with the paid-directory commitment;
these settings alone cannot accept payments. Provider-side currency, recurrence
and publication entitlement policy remain future work.

## Verification

```sh
node scripts/spec-lint.mjs docs/spec
node scripts/verify-foundation.mjs build
node scripts/verify-foundation.mjs database
node scripts/verify-foundation.mjs accounts
node scripts/verify-foundation.mjs ui
node scripts/verify-foundation.mjs setup
node scripts/verify-provider-administration.mjs
```

The verifiers copy application source into a disposable directory under the sibling
`scratchpads` directory. Set
`FOUNDATION_SCRATCH_DIR` to choose another scratch root. Each run removes its
own disposable directory on completion, including failed checks. It never copies
the checkout's `.env` or existing database. The build check uses committed locks;
the database check runs migration twice against its own SQLite file and compares
the account schema and migration history. Compiled Rust artifacts are cached under
the source checkout's ignored `target` directory. Editing-time checks include new,
unignored source files; Cairn evidence requires committing all declared inputs first.

The database check establishes SQLite behavior only. PostgreSQL behavior has not
been verified. Use `cairn wake` for the next action; receipts and logs remain tracked
under `.cairn/evidence`.

The account check exercises the actual HTTP router and middleware with an isolated
database and a captured mail transport. It checks registration, verification,
login, logout, reset, CSRF rejection, token expiry/reuse/ownership, and session
revocation. It does not establish delivery through an external SMTP service.

The UI and setup checks need Chromium installed once:

```sh
cd frontend
bunx playwright install chromium
cd ..
```

On Linux, install the browser's system dependencies with
`bunx playwright install-deps chromium` from `frontend` if Playwright reports
missing libraries. The UI check runs built assets against a disposable server,
checks permissions and keyboard controls, and changes a shared branding token.
Screenshots are written to the ignored `target/foundation-ui` directory.

The setup check executes the marked README command blocks in a disposable copy,
creates a new local key, migrates the example database, starts Vite and the
application, and opens the public and sign-in pages in Chromium. It also checks
an invalid login returns a visible error. It does not send SMTP mail or establish
external delivery. No operator configuration or database is read or copied.

Both shells use the semantic tokens in `frontend/src/styles/tokens.css`.
Administration requires the explicit `admin.access` permission; an account or
an `administrator` role name alone grants no access.

The provider verifier exercises both real adapters without external requests,
direct HTTP access and CSRF failures, the administrator command, encrypted storage,
wrong/missing/default keys, corrupted ciphertext, concurrent and failed writes,
mapping changes, and browser administration journeys. It captures output to check
for secret markers. These tests use synthetic credentials and establish local
integration only; actual provider authentication and delivery remain unverified.
