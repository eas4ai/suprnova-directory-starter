# Suprnova directory starter handoff

Updated: 2026-10-18 (America/New_York)
Origin: https://github.com/eas4ai/suprnova-directory-starter.git

## Current contract

Read `AGENTS.md`, `docs/spec/glossary.md`, `docs/spec/overview.md`,
`docs/spec/roadmap.md` and `docs/spec/framework-upgrade.md` before acting.
The current commitment is `suprnova-4-2`. It preserves all 55 delivered
requirements and adds six upgrade requirements. Traffic reporting remains planned.
Do not restart product discovery or add neighboring features.

This checkout's initialized referee is Sudus. Its receipts and current verdict,
not this handoff, establish completion. Use the same declared environment when
running checks and inspecting freshness, including `CARGO_BUILD_JOBS` and `TMPDIR`.
Legacy Cairn records remain in Git history. Follow the named action and stop on
an escalation or Done.

## Dependency and migration boundaries

Framework, Stripe and Paddle select tag `v4.2.0` at
`6f6f83feae1dfde97d614d083625872987407780`. Markdown v0.4.0 is pinned to
`6f27ecae53babc0fd637c23b41bc5b244543c78e` and shares that framework identity.
Inertia Vue, core and Vite are pinned to 3.8.0; Vuetify 0 remains 1.0.1.
Cargo and Bun locks are committed. Application package versions remain 1.0.1;
the old starter `v1.0.1` tag still identifies its earlier 3.2.1 release.

The registered framework migrations convert the old RBAC layout and database
sessions. Keep signed application account IDs. `tests/framework_upgrade_migration.rs`
uses a real 3.2.1 schema fixture and checks records, grants, operator recovery and
repeat migration. Never run old binaries on the migrated database. UPGRADE.md
explains backups, sign-in expectations and restoring matching code/data/media/key.

## Application boundaries

- `src/accounts` wraps framework authentication and forwards credential rehash.
  The upgrade tests cover suspension, CSRF rotation and stale-session rejection.
- `src/listings` and `src/articles` separate saved revisions from approved public
  content. Publication eligibility also depends on moderation, account state and
  payment entitlement. Markdown resolves the current eligible revision each time.
- `src/billing` owns provider configuration, immutable purchase terms, verified
  payment evidence and retryable fulfillment. Stripe/Paddle adapters own provider
  protocol behavior. Application verification uses synthetic credentials/events.
- Shared bootstrap initializes services. HTTP bootstrap installs middleware and
  Inertia only for the server; console and queue workers need no frontend manifest.
- Vite and Inertia explicitly include `src/app.css`. The Inertia Vite plugin serves
  hot SSR from source; production runs the built SSR worker separately.
  HTML, Inertia and cookie-bearing responses are `private, no-store`.
- Owner notification intents and payment reconciliation use their existing bounded
  commands. Framework credential-revocation retries additionally require a durable
  Redis queue and a supervised `directory queue:work` process in production.

Do not restore cached CSRF meta-token overrides, a second permission engine or
provider behavior inferred from browser-return parameters. The product remains
an MIT source starter. Larafast Directories guided behavior; its purchased source
and assets must not enter the distribution. THIRD_PARTY_NOTICES.md records reused
MIT material and fixtures.

## Verification and operation

README.md is the fresh-install and operations guide. Its marked command blocks
are executed in disposable installations. UPGRADE.md is for existing operators;
do not use fresh-install key generation, demo seeding or database replacement
as upgrade steps.

Run `node scripts/verify-framework-upgrade.mjs` with one of `build`, `migration`,
`auth`, `http`, `console` or `regression`. The last runs every inherited acceptance
mechanism. Separate scripts retain their existing requirement ownership.
Browser checks use Chromium, real Vue SSR and disposable data. Console checks
inject a command only into a disposable copy to prove custom exit-code forwarding.

Snapshots copy application source into the sibling `scratchpads` directory;
`FOUNDATION_SCRATCH_DIR` can override that location. The ignored `target` directory
is a build cache. Never read, copy or mutate operator `.env`, databases, private
media or provider credentials for acceptance. Each verifier owns and removes its
own temporary processes and data.

Local passes establish SQLite and synthetic-provider behavior. PostgreSQL,
external SMTP, Redis persistence/outage recovery, cloud storage, actual payment
accounts and production TLS/proxy deployment require operator smoke tests.
No deployment or new application release tag is implied by the upgrade work.
