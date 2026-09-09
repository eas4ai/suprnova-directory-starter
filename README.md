# Suprnova directory starter

A free directory starter built on Suprnova. The current commitment establishes
the application foundation. Listing publication and Stripe/Paddle administration
are planned in [the roadmap](docs/spec/roadmap.md).

## Prerequisites

- Rust 1.94 or newer and Cargo; this checkout is verified with Rust 1.95.
- Bun 1.4.1 for frontend packages and database inspection.
- Node 24 and an installed Cairn checkout for the verification workflow.
- A C/C++ toolchain, pkg-config and OpenSSL development headers for native dependencies.
- Git and network access for the first dependency installation.

## Install and migrate

Run from a fresh checkout. Keep an existing operator configuration when updating
an installation; the copy command below is only for a new checkout.

```sh
cp .env.example .env
cargo build --locked --bins
cd frontend
bun install --frozen-lockfile
bun run build
cd ..
cargo run --locked --bin directory -- migrate
```

The default database is `database.db` in this checkout. SQLite requires no database
service. The migration command creates the account tables and records completed
migrations. Running the same command again applies only pending migrations.
It is Suprnova's application migration entry point; the developer CLI's
`suprnova migrate` delegates to it.

Account email uses the SMTP service on port 1025 from the example configuration.
Registration sends a verification link; sign in and open `/verify-email` to resend
it. Open the link while signed in to the account it belongs to. Password reset
at `/forgot-password` sends links only for verified accounts. Unknown and unverified
addresses receive the same public response. Resetting a password revokes existing
sessions and remember tokens. Mail links use `APP_URL`, so set it to the address
where the application is reachable.

## Verification

```sh
node scripts/spec-lint.mjs docs/spec
node scripts/verify-foundation.mjs build
node scripts/verify-foundation.mjs database
node scripts/verify-foundation.mjs accounts
node scripts/verify-foundation.mjs ui
```

The verifier copies tracked source into a disposable directory. It never copies
the checkout's `.env` or existing database. The build check uses committed locks;
the database check runs migration twice against its own SQLite file and compares
the account schema and migration history. Compiled Rust artifacts are cached under
the source checkout's ignored `target` directory. Stage newly added source before
an editing-time check; Cairn evidence requires committing all declared inputs first.

The database check establishes SQLite behavior only. PostgreSQL behavior has not
been verified. Use `cairn wake` for the next action; receipts and logs remain tracked
under `.cairn/evidence`.

The account check exercises the actual HTTP router and middleware with an isolated
database and a captured mail transport. It checks registration, verification,
login, logout, reset, CSRF rejection, token expiry/reuse/ownership, and session
revocation. It does not establish delivery through an external SMTP service.

The UI check needs Chromium installed once with `cd frontend && bunx playwright
install chromium` (then return to the repository root). On Linux, install the
browser's system dependencies with `bunx playwright install-deps chromium` if
Playwright reports missing libraries. It runs the built frontend against a
local disposable server, checks permission denial, keyboard navigation and dialogs,
and changes a shared branding token across both shells. Screenshots are written
to the ignored `target/foundation-ui` directory.

Both shells use the semantic tokens in `frontend/src/styles/tokens.css`.
Administration requires the explicit `admin.access` permission; an account or
an `administrator` role name alone grants no access.
