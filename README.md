# Suprnova directory starter

A free directory starter built on Suprnova, with owner submissions, moderation,
directory search and free or paid publication through Stripe and Paddle.
See [the roadmap](docs/spec/roadmap.md) for the remaining starter work.

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

The command grants `admin.access`, `billing.configure` and `listings.moderate` using Suprnova RBAC.
It refuses unknown or unverified accounts. Repeating a grant is safe. Sign in and
open **Administration → Payment providers**. An existing session sees the grant
on its next request; registration itself never grants administrative access.

To revoke access, including access in an existing session:

```sh
cargo run --locked --bin console -- admin:access revoke --user-id 42
```

Revocation removes these three direct permissions and any role memberships granting
any of them. Other role memberships remain. Roles and their permissions
are not deleted. The same host command can recover access if no administrators
remain; no public bootstrap endpoint is exposed.

## Listings and moderation

After migrating, create the initial categories:

```sh
cargo run --locked --bin console -- directory:categories
```

Repeated runs preserve existing terms. Verified owners use **Your listings** to
save drafts, upload an optional image, and submit for review. Administrators use
**Listing reviews** to approve or reject the exact submitted revision. Rejected
owners see the reason and can save a revised draft before resubmitting. Existing
approved content stays public during editing when publication eligibility remains
valid. After approval, owners choose a publishing plan. Approval alone does not
grant publication. Free plans grant eligibility immediately; paid plans wait for
verified provider evidence.

Images are decoded and re-encoded as PNG, with a 5 MiB input limit and maximum
dimensions of 4096 × 4096. Set `DIRECTORY_MEDIA_ROOT` to a private local directory
outside `public/`; the default is `storage/private/directory`. The process needs
write access. Back up that directory with the database. Draft images require owner
or moderator access, and public image routes check current listing eligibility.

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
Create the corresponding local plan under **Publishing plans**. Choose free,
one-time, monthly or annual billing, enter the advertised amount in minor units
and currency, and enable it. Existing purchases retain their original terms when
plans, mappings or provider configuration change.

## Paid publication and recovery

Set `BILLING_CHECKOUT_MODE=test` or `live` explicitly. Only the selected profile
is offered to owners; test payments never appear in the public directory.
Configure the matching provider webhook destination:

| Provider | Test path | Live path |
| --- | --- | --- |
| Stripe | `/billing/webhooks/stripe/test` | `/billing/webhooks/stripe/live` |
| Paddle | `/billing/webhooks/paddle/test` | `/billing/webhooks/paddle/live` |

Use your HTTPS application origin before each path. Supply each destination's
signing secret in its matching profile. Subscribe to checkout/transaction,
invoice, subscription, refund and dispute/adjustment events relevant to your
provider. The endpoints authenticate and retain evidence, then return 202;
acceptance does not mean fulfillment succeeded.

Run the following from a scheduler every minute, with the same database and
`APP_KEY` as the web process. It processes at most 25 due events, with a 120-second
batch deadline and a 90-second per-event deadline. Expired worker leases are
recoverable. Automatic retries stop after eight failed attempts.

```sh
cargo run --locked --bin console -- billing:reconcile --limit 25
cargo run --locked --bin console -- billing:reconcile --status --limit 100
```

Monitor command failures and unresolved event IDs. Correct a price, currency,
customer, settlement or credential problem at its source, then retry the retained
event. The `--event` value is the local event ID printed by the status command:

```sh
cargo run --locked --bin console -- billing:reconcile --event EVENT_ID
cargo run --locked --bin console -- billing:reconcile --purchase PURCHASE_ID
```

A missing webhook can be recovered from the purchase's saved session or
subscription. If the create response was lost, look up its `purchase_id`
metadata/custom data in the provider dashboard and supply the existing resource:

```sh
cargo run --locked --bin console -- billing:reconcile --purchase PURCHASE_ID --resource PROVIDER_RESOURCE_ID
```

Recovery only reads existing provider state; it never creates charges. Stripe
checkout retries retain the original idempotency key within the supported window.
An ambiguous Paddle create remains pending: do not create another transaction to
work around a lost response. Recover the original transaction by correlation.
Browser return parameters cannot grant publication.

One-time payment has no expiry. Recurring access ends at the last verified paid
period, without a grace period for failed renewal. Owner cancellation requests
stop future renewals only after provider confirmation; scheduled cancellation
retains the paid period. Full refunds remove the affected payment's eligibility,
partial refunds preserve it, open disputes suspend it, and confirmed wins restore
it subject to moderation. Initiate refunds in the provider dashboard.

Disabling a provider stops new checkout while existing purchases remain
recoverable. Purchases retain encrypted credentials and immutable price terms;
clearing a disabled profile is permitted only when retained recovery material is
readable. Old purchase signing keys and the currently configured key can overlap
during replacement. Authenticated retained events replay without rejecting their
now-old signature timestamps. If the old API key is revoked, install the new key
in the same account/mode and explicitly select it for read-only recovery:

```sh
cargo run --locked --bin console -- billing:reconcile --purchase PURCHASE_ID --use-current-credentials
```

Do not change provider accounts to recover an existing purchase. Keep the
database and matching `APP_KEY` together in backups; clearing current settings
does not erase the encrypted material retained with purchases. Raw authenticated
events remain private database records and may contain provider customer data.

Local verification uses synthetic provider responses and the actual pinned
adapters' signature checks. Before accepting real money, use operator-owned
sandbox accounts and matching prices to exercise an approved test listing,
checkout, actual webhook delivery, reconciliation, renewal/cancellation and
refund/dispute outcomes. Confirm test listings stay private. This external smoke
test is separate from the local suite and has not been performed by the starter's
local verifier.

## Verification

```sh
node scripts/spec-lint.mjs docs/spec
node scripts/verify-foundation.mjs build
node scripts/verify-foundation.mjs database
node scripts/verify-foundation.mjs accounts
node scripts/verify-foundation.mjs ui
node scripts/verify-foundation.mjs setup
node scripts/verify-provider-administration.mjs
node scripts/verify-paid-directory.mjs directory
node scripts/verify-paid-directory.mjs payments
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

The paid-directory verifier exercises signed ingress, rejected evidence,
transaction interruption, replay, renewals, refunds, disputes and recovery in both
modes. It runs local HTTP contracts for the pinned SDKs and checkout adapters, then
browser journeys for plans and purchases. Payment screenshots go to
`target/payments-ui`; directory screenshots go to `target/directory-ui`.
