# Suprnova directory starter

A free directory starter built on Suprnova. The current commitment establishes
the application foundation. Listing publication and Stripe/Paddle administration
are planned in [the roadmap](docs/spec/roadmap.md).

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
service. The migration command creates the account tables and records completed
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

## Verification

```sh
node scripts/spec-lint.mjs docs/spec
node scripts/verify-foundation.mjs build
node scripts/verify-foundation.mjs database
node scripts/verify-foundation.mjs accounts
node scripts/verify-foundation.mjs ui
node scripts/verify-foundation.mjs setup
```

The verifier copies tracked source into a disposable directory under the sibling
`scratchpads` directory. Set
`FOUNDATION_SCRATCH_DIR` to choose another scratch root. Each run removes its
own disposable directory on completion, including failed checks. It never copies
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
