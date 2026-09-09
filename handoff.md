# Suprnova directory starter handoff

Updated: 2026-09-09 (America/New_York)
Working repository: `/home/shawn/workspace2/suprnova-directory-starter`
Origin: https://github.com/eas4ai/suprnova-directory-starter.git

## Current contract

This repository uses Cairn. Read `AGENTS.md`, run `cairn wake`, and follow the
named action. `docs/spec/roadmap.md` names `provider-administration` as Current.
The developer confirmed PAY-001 and PAY-004 through PAY-008, their falsifiers and
operating rules. The foundation commitment is complete. PAY-002 and PAY-003 and
unresolved payment policies remain Draft.

Do not restart discovery or ask the developer to repeat settled choices.
Read `docs/spec/glossary.md`, `docs/spec/payments.md`, the roadmap,
`docs/commitments/provider-administration.md`, and its decision records first.
Cairn receipts and their output are tracked under `.cairn/evidence`; freshness
and the final review determine completion, not this handoff's prose.

## Implemented foundation

- Rust binaries pin Suprnova v1.3.7; Cargo and Bun dependency locks are committed.
- The example SQLite database migrates through the application command. Repeated
  migration preserves schema, history and existing account data. RBAC tables
  use Suprnova's migration.
- Framework-backed registration, login, logout, email verification/resend and
  password reset have HTTP journeys with denied actors, CSRF and token failures.
  Password resets require a verified address and revoke existing credentials.
- Vue/Inertia public and administration shells share semantic CSS tokens and
  components. Vuetify 0 is pinned to 1.0.1. Administration requires `admin.access`
  for the `directory.user` model type; neither registration nor a role name
  grants it. Directory/listing content is an honest empty state.
- The local setup guide supplies a generated local key, install/migration commands,
  and separate Vite/application terminal commands. The setup verifier executes
  those README blocks and checks rendering plus a visible invalid-login error.

The frontend uses Inertia's current XSRF cookie support. Do not reintroduce the
old cached meta-token override, or explicit `errors: null` login/register props;
browser checks exposed both as defects. No custom modal focus workaround is
needed: wait for Inertia navigation before testing the destination controls.

## Verification and limits

`README.md` is the setup and verification guide. The five commands are
`node scripts/verify-foundation.mjs build|database|accounts|ui|setup`, one task
per invocation. `node scripts/spec-lint.mjs docs/spec` uses the installed Cairn
checkout. Chromium is required for browser checks; installation is documented.

Checks copy tracked source into an isolated directory under the sibling
`scratchpads` directory: `/home/shawn/workspace2/scratchpads/` on this machine.
Set `FOUNDATION_SCRATCH_DIR` to override it. Each check cleans up its own copy,
processes and temporary browser data. The source checkout's ignored `target`
is a compiled-artifact cache. Never copy or modify its operator `.env` or database.
Editing-time snapshots include new unignored source files; Cairn checks require commits.

## Provider administration work

Provider administration is implemented; use Cairn evidence and the final review for
verification status. `src/billing` owns versioned test/live settings, secret protection, local
validation and adapter resolution. One conditional database update commits each
mode's profile/default/mapping changes atomically. `src/commands/admin_access.rs`
uses Suprnova RBAC entities to grant/revoke the starter permission bundle.
`frontend/src/pages/admin/Billing.vue` edits configuration through protected Inertia
requests. The README documents provisioning, configuration and recovery limits.

Run `node scripts/verify-provider-administration.mjs` for the local contract and
browser checks. The verifier creates disposable data and synthetic credentials.
Never substitute real provider credentials. No external provider authentication,
price lookup, checkout or webhook delivery is established by this commitment.

SQLite is verified; PostgreSQL is not. Account tests capture mail inside the
process. Setup renders forms without sending email. External SMTP delivery,
production deployment, runtime SSR, payments and listing features are not
established by these passes. For local account mail, provide the SMTP capture
server declared in README and `.env.example`.

The foundation's final review is `.cairn/reviews/starter-foundation.md`; its notes
about future administrator provisioning describe that earlier commitment. Provider
failure demonstrations are in `docs/provider-administration-mechanism-review.md`.
The provider final review belongs in `.cairn/reviews/provider-administration.md`. Earlier
entries record failures and corrections as historical evidence. Use `cairn wake`
for current requirement status and the next action. Stop when this commitment
is Done; the developer chooses the next commitment.

## Settled product direction

The product is a free MIT source starter that operators can monetize. The first
full delivery includes owner listings, categories, bounded discovery, moderation,
paid publishing, editorial content, taxonomy, RSS, SEO, administration,
notifications, storage, demo data and adoption documentation. This foundation
commitment does not authorize implementing those later modules.

Use Suprnova's authentication, RBAC and payment adapters rather than parallel
engines. Stripe and Paddle are both selected for provider administration.
The selected lifecycle is submit, review/approve, then checkout. Detailed plan,
renewal, refund, dispute and entitlement policies need agreed requirements.
Lemon Squeezy is not a shipped Suprnova adapter.

NOWPayments is a separate planned framework workstream for the next framework
release; see `docs/commitments/nowpayments-framework-adapter.md`. Nation X is
implementation evidence. Do not modify sibling projects under this commitment.
Self-hosted Keygen and future AI features remain brainstorming only.

## Discovery records

- `docs/discrepancies.md`: observed integration, harness and CLI discrepancies,
  corrections and remaining verification limits.

- `docs/feature-map.md`: product baseline and proposed acceptance checks.
- `docs/architecture-proposal.md`: proposed module and billing responsibilities.
- `docs/pulsar-assessment.md`: Pulsar reuse evidence and selected UI.
- `docs/recon.md`: initial inspection, with historical verification limits.
- `../suprnova`: development source; inspect the exact v1.3.7 tag for released APIs.
- `../Pulsar`: MIT application reference on an older framework/Vuetify version.
- `../suprnova.app`: reference for Vue/Inertia/Vuetify 0 composition.
- `../larafast-directories-master`: purchased legacy reference, not a contract
  or codebase to copy wholesale.

The initial recon limitations describe that earlier inspection, not current
runtime evidence. No sibling test pass or deployed-production claim is implied.
Apply the imported machine production rules. Use graph discovery first, focused
source reads when graph metadata is stale, and rtk-prefixed shell commands.
The developer asked for scratchpads in the workspace and cleanup after each run.
