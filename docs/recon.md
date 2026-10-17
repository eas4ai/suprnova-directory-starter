# Directory starter reconnaissance

Status: Observed
Date: 2026-09-08

The developer named this repository as the destination for a reusable directory starter built with Suprnova. The purchased Laravel application supplies feature references. Its observed behavior is not automatically the contract for this application.

## Exists

| Finding | Evidence |
| --- | --- |
| The package is directory, described as a Suprnova directory starter. It requires Rust 1.94.0. Initially pinned to v1.3.5, it now pins v1.3.7 following developer direction. | `Cargo.toml:1`, `Cargo.toml:29`. |
| The frontend uses Vue 3, Inertia and Tailwind. Build performs Vue type checking before Vite; separate type-check and SSR build commands exist. | `frontend/package.json:6`. |
| The server registers configuration, shared bootstrap, HTTP bootstrap, routes and migrations. A separate console binary is declared. | `cmd/main.rs:8`, `Cargo.toml:25`. |
| Routes cover a home page, guest login/registration, authenticated dashboard and POST logout. No directory or moderation route is declared here. | `src/routes.rs:6`. |
| Registration checks duplicate email, creates a user, logs them in and redirects to the dashboard. The user model hashes passwords and declares fillable/hidden fields. | `src/controllers/auth.rs:115`, `src/models/user.rs:18`, `src/models/user.rs:59`. |
| The user model implements email-verification and password-reset traits. Those traits do not establish complete user flows: the current application route table has no corresponding endpoints. | `src/models/user.rs:124`, `src/models/user.rs:146`, `src/routes.rs:6`. |
| Registered migrations cover users, sessions, remember tokens and auth-flow tokens. No category or listing migration is registered. | `src/migrations/mod.rs:12`. |
| HTTP bootstrap wires logging, sessions, Vue Inertia, locale, CSRF, include-field parsing and shared locale data. | `src/bootstrap.rs:84`. |
| Home is a starter welcome page. Application pages discovered are Home, Dashboard, Login and Register. | `frontend/src/pages/Home.vue:8`, `frontend/src/pages/Dashboard.vue`, `frontend/src/pages/auth/Login.vue`, `frontend/src/pages/auth/Register.vue`. |

## Documented

| Finding | Evidence |
| --- | --- |
| Source comments explain authentication, model generation and middleware order. | `src/models/user.rs:1`, `src/bootstrap.rs:70`. |
| The ignore file documents tracking Cargo.lock and excluding runtime environment files and mail previews. | `.gitignore:6`, `.gitignore:21`, `.gitignore:128`. |
| No Markdown documentation or existing spec set was found before this recon. | Inventory record below. |

## Contradicted

| Finding | Both sides |
| --- | --- |
| The ignore-file commentary says the Dockerfile copies Cargo.lock, but neither file is present in this scaffold. This appears to be inherited scaffold documentation, not an established deployment procedure. | `.gitignore:6`; inventory record below. |

No Agreed requirements exist to assess contractual drift. No previous target recon existed to carry forward. Unresolved findings in the reference application's report remain there; this report does not close them.

## Unverified

| Finding | Evidence / next check |
| --- | --- |
| Runtime and compilation are unverified. No dependency lockfiles or installed frontend dependencies were found. | Inventory below; `Cargo.toml:29`, `frontend/package.json:6`. |
| Initial history was absent at first inspection; the developer subsequently committed and published the scaffold. Local HEAD and remote main now match. Only the new docs are untracked. | Follow-up Git record below. |
| No tests directory, scripts directory or .github directory was found. Inline tests and framework tests are not a substitute for verified directory behavior. | Inventory below. |
| No project-specific working agreement, production rules override or Cairn roadmap was found. | Inventory and wake result below. |
| Payment provider, entitlement rules, moderation lifecycle and starter customization contract remain undecided. | Developer requested a directory starter on 2026-09-08; proposed scope is in `docs/feature-map.md`. |

## Inspection record

Read-only inspection on 2026-09-08:

- Graph indexing and source inspection completed for this repository.
- `git status --short` showed the existing scaffold files as untracked. `git log -5 --oneline` reported that main has no commits yet.
- Markdown discovery returned no files before this report.
- Existence checks found no Cargo.lock, frontend/package-lock.json, frontend/bun.lock, frontend/node_modules, tests, scripts, .github, AGENTS.md, BEST_PRACTICES.md or docs/spec/overview.md. Root inventory contained no Dockerfile.
- `cairn wake` exited 3: not a Cairn repository, no docs/spec/roadmap.md.
- No application build, tests, migration, dependency install or service startup was attempted during this scope-definition pass. The spec lint script is absent.
- The existing .env was not read or changed.

Follow-up after the developer created the GitHub origin on 2026-09-08:

- Origin is `https://github.com/eas4ai/suprnova-directory-starter.git`.
- `git ls-remote --heads origin` succeeded and returned main at `a49cb8e841c6124381e12e7dea89ac544cee0916`.
- `git rev-parse HEAD` returned the same commit; `git log -3` names it `first commit`.
- `git status --short --branch` showed main tracking origin/main with only docs/ untracked. This supersedes the initial uncommitted-scaffold observation above.

## Todo

- Complete: inspect the target scaffold and feature gaps; verify cited source locations.
- Complete: record target recon and proposed feature scope.
- In progress: settle the first commitment's scope and acceptance checks with the developer.
- Pending: write requirements and their falsifiers, obtain agreement, then prepare the commitment and its mechanisms.


## Cairn mechanism archive — 2026-10-17

Documented: these declarations are copied verbatim from commit `3d568ec154d1f6bcd1015225e9546450249d66ba` before migration. They preserve the commands, declared inputs and requirement mapping; they are not fresh Sudus evidence.

### admin-overview

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/admin-overview`.

```yaml
command: node scripts/verify-complete-starter.mjs overview
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - OVR-001
  - OVR-002
  - OVR-003
```

### delegated-administration

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/delegated-administration`.

```yaml
command: node scripts/verify-complete-starter.mjs administration
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - ADM-001
  - ADM-002
  - ADM-003
```

### directory-workflows

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/directory-workflows`.

```yaml
command: node scripts/verify-paid-directory.mjs directory
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - DIR-001
  - DIR-002
  - DIR-003
  - DIR-004
  - DIR-005
  - DIR-006
  - DIR-007
```

### editorial-publication

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/editorial-publication`.

```yaml
command: node scripts/verify-complete-starter.mjs editorial
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - CNT-001
  - CNT-002
  - CNT-003
  - CNT-004
  - CNT-005
  - CNT-006
```

### foundation-accounts

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-accounts`.

```yaml
command: node scripts/verify-foundation.mjs accounts
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - FND-003
```

### foundation-build

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-build`.

```yaml
command: node scripts/verify-foundation.mjs build
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - .env.example
requirements:
  - FND-001
reviewed:
  - FND-001 sha256:cab1d772d0a6216adaec8e1b1fa93bc3e8c9d2199843736579fda8969d488ab2
  - FND-001 sha256:6b895667a44f2b9e4492727a51f67211445bb32d91c43b6f3dfd914b5478a67c
  - FND-001 sha256:736d84250da2c2b6d70a8aadcc8038ee6fdcc1be05f1d4ecf77f62413cbf3c43
```

### foundation-database

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-database`.

```yaml
command: node scripts/verify-foundation.mjs database
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - .env.example
requirements:
  - FND-002
```

### foundation-setup

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-setup`.

```yaml
command: node scripts/verify-foundation.mjs setup
results: per-requirement
inputs:
  - .gitignore
  - handoff.md
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - .env.example
requirements:
  - FND-005
```

### foundation-ui

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-ui`.

```yaml
command: node scripts/verify-foundation.mjs ui
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - FND-004
```

### paid-lifecycle

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/paid-lifecycle`.

```yaml
command: node scripts/verify-paid-directory.mjs payments
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - DIR-008
  - PAY-002
  - PAY-003
  - PAY-009
  - PAY-010
  - PAY-011
  - PAY-012
  - PAY-013
  - PAY-014
  - PAY-015
  - PAY-016
```

### provider-administration

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/provider-administration`.

```yaml
command: node scripts/verify-provider-administration.mjs
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - PAY-001
  - PAY-004
  - PAY-005
  - PAY-006
  - PAY-007
  - PAY-008
reviewed:
  - PAY-001 sha256:88cb0a671a89e6443adcb96e5edf94ec2351a9cdf32a2536dec94a3925e302c3
  - PAY-001 sha256:1af61b53911c61f25abcfe20021aa6c54ac13c2dc77e580db1e54d0a2d404d19
  - PAY-001 sha256:47a350feb1d4eef45f36d3a14e93696635fef8dcb4b62909edcb9088ba9e51e4
```

### seo-controls

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/seo-controls`.

```yaml
command: node scripts/verify-complete-starter.mjs seo
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - SEO-001
  - SEO-002
  - SEO-003
  - SEO-004
  - SEO-005
  - SEO-006
reviewed:
  - SEO-001 sha256:7805abeb592222e448f738704c52b7305c14bbae1918ea2a8ce9f81f2baa182a
```

### site-appearance

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/site-appearance`.

```yaml
command: node scripts/verify-appearance.mjs
results: per-requirement
inputs:
  - Cargo.toml
  - Cargo.lock
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
  - README.md
requirements:
  - UI-001
  - UI-002
```

### starter-adoption

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/starter-adoption`.

```yaml
command: node scripts/verify-complete-starter.mjs adoption
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - LICENSE
  - THIRD_PARTY_NOTICES.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - KIT-001
  - KIT-002
  - KIT-003
  - KIT-004
  - KIT-005
```

### upgrade-guidance

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/upgrade-guidance`.

```yaml
command: node .cairn/checks/verify-upgrade-guide.mjs
results: per-requirement
inputs:
  - UPGRADE.md
  - Cargo.toml
  - Cargo.lock
  - frontend/package.json
  - README.md
  - .cairn/checks/verify-upgrade-guide.mjs
requirements:
  - UPG-001
```


## Suprnova 4.2 upgrade reconnaissance — 2026-10-17

This section supersedes the scaffold observations above for the current upgrade.
The inspected application baseline is `d1eb68a`; application code is unchanged
from `3d568ec`. Historical passing records are not fresh upgrade evidence.

| State | Finding and evidence |
| --- | --- |
| Exists | Starter 1.0.1 pins Suprnova and both payment adapters to 3.2.1 revision `2bd4bd53d04fa4581fdbb152ebd70cfb682b342c`; committed Rust and Bun locks exist (`Cargo.toml`, `Cargo.lock`, `frontend/bun.lock`). |
| Exists | Accounts, owner submissions, moderation, payments, articles, administration, appearance and SEO have implementation and acceptance scripts (`src/routes.rs`, `scripts/verify-foundation.mjs`, `scripts/verify-paid-directory.mjs`, `scripts/verify-complete-starter.mjs`, `frontend/tests/`). This closes the earlier observation that these files were absent, not a new runtime verification. |
| Exists | Account permissions use old RBAC pivot names in direct SQL and entity operations; role/permission IDs are signed and display_name is written (`src/accounts/queries.rs:15`, `src/accounts/roles.rs:8`). These call sites need adaptation to the 4.0 RBAC schema. |
| Exists | The migrator registers application-owned sessions and framework CreateRbacTables without the new compatibility migrations (`src/migrations/mod.rs:25`). Existing populated databases require coverage in addition to fresh installation. |
| Exists | AccountUserProvider forwards credential checks and password changes but has no rehash forwarding hook (`src/accounts/provider.rs:42`). Console errors collapse to exit code 1 (`src/bin/console.rs:49`). |
| Exists | Inertia installs the default Vue configuration, CSS is imported from JavaScript, and Vite has one client entry (`src/bootstrap.rs:143`, `frontend/src/main.ts:3`, `frontend/vite.config.ts:35`). Separate SSR source and build already exist (`frontend/src/ssr.ts`, `frontend/package.json`). |
| Exists | The Markdown wrapper checks current publication eligibility, restricts methods, forces no-store, and distinguishes lookup failures (`src/seo/markdown.rs`). These protections must survive the plugin API upgrade. |
| Documented | The 4.0 guide calls for explicit environment configuration, session/RBAC compatibility migrations, current CSRF cookies, auth-provider adaptation and console exit-code propagation: https://suprnova.app/docs/upgrade-4-0. The 4.1 guide describes optional security-header/CSP and Sentry integration: https://suprnova.app/docs/upgrade-4-1. The 4.2 guide requires explicit CSS entries and documents cache/cookie behavior: https://suprnova.app/docs/upgrade-4-2. Guides were fetched on 2026-10-17. |
| Exists | Remote Suprnova tag v4.2.0 resolves to `6f6f83feae1dfde97d614d083625872987407780` (git ls-remote). The latest remote Markdown release v0.3.0 resolves to `14cb9e0756ea032f8117bcc1309bf4f7ef838d8c`; its manifest selects Suprnova tag v4.1.0 (`../suprnova-markdown/Cargo.toml`; remote HEAD matches). A published compatible Markdown revision is a prerequisite; mixing framework source identities does not satisfy the contract. |
| Documented | Foundation, payment and SEO specifications intentionally pin the prior versions (`docs/spec/foundation.md`, `payments.md`, `seo-controls.md`). Updating these pins is the requested change, not evidence of existing code/spec drift. All 55 delivered requirements remain acceptance obligations; the eight planned traffic requirements stay outside this upgrade (`docs/spec/roadmap.md`). |
| Documented | History since the newest Agreed date includes the 3.2.1 upgrade, native RBAC timestamp repair, Markdown obligations, release review and upgrade guide, followed by the authorized Sudus migration (`git log --since=2026-10-05`; commits `1cb875d`, `512becc`, `49a06da`, `89700be`, `689198a`, `0713af1`, `d1eb68a`). |
| Contradicted | The supplied working agreement still names Cairn 1.x, but `cairn wake` is unavailable and initialized Sudus reports no commitment started. Replace the agreement only through the existing-project authorization step. |
| Unverified | No 4.2 build, fresh migration, populated migration, browser test or adapter test has run. PostgreSQL, real provider accounts, external SMTP and deployment remain outside local SQLite acceptance (`docs/spec/adoption.md`). |
| Unverified | Earlier operational findings in `docs/discrepancies.md` remain historical until their corresponding fresh checks run. The carried license/admin backlog item is preserved as `ec98b25a55f5562c80973859b7d001e7f243ed43`; its presence is not a current defect claim. |

### Upgrade radius and proposed verification

- FND: manifests/locks, shared and HTTP bootstrap, migrations, authentication,
  console, frontend entries and existing foundation checks. Preserve current
  public/admin behavior while adopting documented framework changes.
- ADM/PAY: RBAC entities and direct SQL, operator grant/revoke, last-admin
  protection, signed account IDs, provider adapters and immutable payment state.
- DIR/CNT/SEO/UI/OVR/KIT: rerun existing workflow, SSR/browser, publication,
  notifications, clean-install and documentation mechanisms. Preserve private
  content and Markdown eligibility boundaries.
- UPG: update backup, migration, worker and rollback instructions to match the
  final pins. Never exercise the operator's untracked database or environment.
- New FWU checks: populated 3.2.1 migration, session security, real served assets
  and cache headers, console exit status, and single framework identity.

No fresh runtime result is asserted by this reconnaissance. The existing
requirements hold as the intended contract; their 4.2 implementation remains
unverified. Optional search/recommendations, Sentry and a new CSP policy are not
part of this compatibility upgrade.


### Markdown prerequisite completed — 2026-10-17

Exists: after explicit developer approval, `suprnova-markdown` v0.4.0 was
committed and published at `6f27ecae53babc0fd637c23b41bc5b244543c78e`.
Remote main and the peeled release tag were verified with `git ls-remote`.
Its manifest uses Suprnova tag v4.2.0; library source and API are unchanged.

Exists: `cargo fmt --check`, `cargo test --locked` (4 unit, 6 middleware,
1 doctest), `cargo check --locked --lib`, guide example syntax checks and
Ripwire quality/test gates passed in the plugin checkout. A separate consumer
metadata graph resolved exactly one Suprnova 4.2 identity without default or
testing features. The plugin working tree was clean after publication.

Unverified: starter runtime integration and acceptance remain pending. The
plugin results do not establish that the starter's migration or workflows pass.


### Sudus contract preparation and mechanism review — 2026-10-17

Documented: the developer confirmed the six FWU requirements and the Markdown
prerequisite. The roadmap now names `suprnova-4-2` with 61 requirements: the
55 delivered contracts and six compatibility obligations. FND-001, PAY-001,
the overview and SEO operating rules now select the approved framework and
published plugin revisions. The historical documentation-only scope is labeled
as historical rather than contradicting this upgrade.

Exists: 21 mechanisms have been declared. Each has an observed failing example,
and all 61 requirement review bindings were checked against their current text.
The old mechanism commands and test suites were retained; the upgrade-guide
check moved from the reserved `.cairn/checks` path into `scripts`. The foundation
build check now inspects resolved dependency identities as well as direct pins.
Its first full-metadata run hit Node's buffer limit; a bounded 32 MiB capture
fixed that harness failure, and the next run reached the intended pin assertion.
The buffer error is not counted as a failure-sensitivity demonstration.

The demonstrations below each test a representative violation of their group.
A grouped receipt marks the group failed; it does not mean every falsifier was
individually mutated. Output was inspected for the intended assertion rather
than accepting a compiler failure, timeout, or missing executable. Every
mutation was restored immediately after its check. These are failure-sensitivity
records, not new acceptance passes. Full behavioral checks and the new 4.2
migration, authentication and browser suites remain implementation work.

| Representative | Failing receipt | Changed input | Observed failure |
| --- | --- | --- | --- |
| ADM-002 | `abc4872bbf2a8e5aed90b52b1448ab90851f5c34` | `src/accounts/mod.rs` | panicked at tests/administration_workflows.rs |
| CNT-002 | `d20ad0c21d85f8427734d2b8b6eddd8d7634b482` | `src/articles/queries.rs` | panicked at tests/editorial_workflows.rs |
| DIR-006 | `5601e94d66fe62cd5a7de859ea1faae6bc135dd7` | `src/listings/queries.rs` | panicked at tests/directory_workflows.rs |
| FND-002 | `4caab2bbfa83648c1f9c7ba22645e737dff415bf` | `src/migrations/m20240101_000001_create_users_table.rs` | Missing account column: users.remember_token |
| FND-003 | `311a844f1705a4cb8fbb5deeeac03f2a2c2416e1` | `src/middleware/authenticate.rs` | panicked at tests/foundation_accounts.rs |
| FND-004 | `824862840abeba7bba89a9445253bf1baba27c0e` | `src/routes.rs` | panicked at tests/foundation_ui.rs |
| KIT-004 | `2256ebbe7aa609b303e5be1582fd9fb4c024bc45` | `LICENSE` | Missing MIT license term: MIT License |
| OVR-001 | `4a1b8725e697d48f9f05b2b0f9588ae323494782` | `src/controllers/admin.rs` | panicked at tests/overview_workflows.rs |
| PAY-001 | `f33ef831e9ebb2629ba7344ba50d80f714dcc5e2` | `src/controllers/billing.rs` | panicked at tests/provider_administration.rs |
| PAY-014 | `99125a4dfc0d051f47aa85841137076346210fb0` | `src/billing/reconcile.rs` | panicked at tests/paid_lifecycle.rs |
| SEO-001 | `ead9cdf10a788469d3761e70afc245adb0475380` | `src/seo/settings.rs` | A saved site description |
| UI-002 | `304b6acdde6a721624f4348cc8f2e422a17a6343` | `src/config/site.rs` | test result: FAILED |

Additional observed failures: FND-001 rejects the still-pinned 3.2.1 manifest
(receipt `9a68b0c1f49910b01cb9ea44a4ea355ee075e100`); FND-005 rejects a missing
installation command block (`23c4d5bbf46936bb39a4fcfd49c2124605dfedd0`); UPG-001
rejects a wrong release identifier (`be570295255ce73699ca0b34266fd6c42e7eb55f`).
FWU-001 through FWU-006 reject the current missing 4.2 dependency, RBAC/session
migration, provider rehash hook, explicit CSS entry, console exit-code forwarding
and updated operations guide respectively. Those guards supplement the runtime
checks, which cannot pass until their compatibility implementation is complete.

Exists: the proposed AGENTS.md is copied exactly from the Sudus 4.5.0 working
agreement. Settings retain `origin` as authority, classify source/interface
paths, identify local database/storage as data, and refuse those runtime paths
in network snapshots. Codex is the review harness without a model override.
The operator SQLite database and its journals are now gitignored; their contents
were neither read nor changed. Final authorization remains pending and no
application dependency or runtime source upgrade has been committed.

Observed verification: spec lint, JavaScript syntax checks, diff whitespace and
exact working-agreement/template comparison pass. Ripwire quality delta reports
no regressions. Its test gate exits 4 for two documentation nodes (the README
heading and the historical discrepancies section) without graph-linked tests;
the executable mechanism demonstrations above are the relevant preparation
checks. This is not a claim that the application upgrade passes acceptance.
